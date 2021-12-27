use crate::channel::AudioChannel;
use crate::source::AudioSource;
use bevy::prelude::Handle;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) enum AudioCommand {
    Play(PlayAudioCommandArgs),
    SetVolume(f32),
    SetPanning(f32),
    SetPlaybackRate(f32),
    Stop,
    Pause,
    Resume,
}

pub(crate) struct PlayAudioCommandArgs {
    /// The settings for this Play command.
    pub(crate) settings: PlayAudioSettings,

    /// An instance handle to communicate with the consumer.
    pub(crate) instance_handle: InstanceHandle,
}

pub enum AudioCommandResult {
    Ok,
    Retry,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub(crate) struct PlayAudioSettings {
    pub source: Handle<AudioSource>,
    pub intro_source: Option<Handle<AudioSource>>,
    pub looped: bool,
}

/// Allows you to interact with a playing sound.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InstanceHandle {
    id: u64,
}

static NEXT_INSTANCE_HANDLE_ID: AtomicU64 = AtomicU64::new(0);

impl InstanceHandle {
    fn new() -> InstanceHandle {
        let id = NEXT_INSTANCE_HANDLE_ID.fetch_add(1, Ordering::SeqCst);
        InstanceHandle { id }
    }
}

/// Playback status of a currently playing sound.
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub enum PlaybackState {
    /// The instance is queued
    Queued,
    /// The instance is playing.
    Playing {
        /// Playback position in seconds
        position: f64,
    },
    /// The instance is paused.
    Paused {
        /// Playback position in seconds
        position: f64,
    },
    /// The instance is stopped and cannot be resumed.
    Stopped,
    /// The instance is fading out and will be paused when the fadeout is finished.
    Pausing {
        /// Playback position in seconds
        position: f64,
    },
    /// The instance is fading out and will be stopped when the fadeout is finished.
    Stopping {
        /// Playback position in seconds
        position: f64,
    },
}

impl PlaybackState {
    /// Get the playback position in seconds
    pub fn position(&self) -> Option<f64> {
        match self {
            PlaybackState::Queued | PlaybackState::Stopped => None,
            PlaybackState::Playing { position }
            | PlaybackState::Paused { position }
            | PlaybackState::Pausing { position }
            | PlaybackState::Stopping { position } => Some(*position),
        }
    }
}

/// Bevy Audio Resource
///
/// Use this resource to play and control your audio
/// ```edition2018
/// # use bevy::prelude::*;
/// # use bevy_kira_audio::Audio;
///
/// fn start_audio_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
///     audio.play(asset_server.load("audio.mp3"));
/// }
/// ```
#[derive(Default)]
pub struct Audio {
    pub(crate) commands: RwLock<VecDeque<(AudioCommand, AudioChannel)>>,
    pub(crate) states: HashMap<InstanceHandle, PlaybackState>,
}

impl Audio {
    /// Play audio in the default channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::Audio;
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     audio.play(asset_server.load("audio.mp3"));
    /// }
    /// ```
    pub fn play(&self, audio_source: Handle<AudioSource>) -> InstanceHandle {
        let instance_handle = InstanceHandle::new();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioCommandArgs {
                settings: PlayAudioSettings {
                    source: audio_source,
                    intro_source: None,
                    looped: false,
                },
                instance_handle: instance_handle.clone(),
            }),
            AudioChannel::default(),
        ));

        instance_handle
    }

    /// Play looped audio in the default channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::Audio;
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     audio.play_looped(asset_server.load("audio.mp3"));
    /// }
    /// ```
    pub fn play_looped(&self, audio_source: Handle<AudioSource>) -> InstanceHandle {
        let instance_handle = InstanceHandle::new();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioCommandArgs {
                settings: PlayAudioSettings {
                    source: audio_source,
                    intro_source: None,
                    looped: true,
                },
                instance_handle: instance_handle.clone(),
            }),
            AudioChannel::default(),
        ));

        instance_handle
    }

    /// Play looped audio in the default channel with an intro
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::Audio;
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     audio.play_looped_with_intro(asset_server.load("intro.mp3"), asset_server.load("audio.mp3"));
    /// }
    /// ```
    pub fn play_looped_with_intro(
        &self,
        intro_audio_source: Handle<AudioSource>,
        looped_audio_source: Handle<AudioSource>,
    ) -> InstanceHandle {
        let instance_handle = InstanceHandle::new();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioCommandArgs {
                settings: PlayAudioSettings {
                    source: looped_audio_source,
                    intro_source: Some(intro_audio_source),
                    looped: true,
                },
                instance_handle: instance_handle.clone(),
            }),
            AudioChannel::default(),
        ));

        instance_handle
    }

    /// Stop all audio in the default channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::Audio;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.stop();
    /// }
    /// ```
    pub fn stop(&self) {
        self.commands
            .write()
            .push_front((AudioCommand::Stop, AudioChannel::default()));
    }

    /// Pause all audio in the default channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::Audio;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.pause();
    /// }
    /// ```
    pub fn pause(&self) {
        self.commands
            .write()
            .push_front((AudioCommand::Pause, AudioChannel::default()));
    }

    /// Resume all audio in the default channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::Audio;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.resume();
    /// }
    /// ```
    pub fn resume(&self) {
        self.commands
            .write()
            .push_front((AudioCommand::Resume, AudioChannel::default()));
    }

    /// Set the volume for the default channel
    ///
    /// The default value is 1
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::Audio;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_volume(0.5);
    /// }
    /// ```
    pub fn set_volume(&self, volume: f32) {
        self.commands
            .write()
            .push_front((AudioCommand::SetVolume(volume), AudioChannel::default()));
    }

    /// Set panning for the default channel
    ///
    /// The default value is 0.5
    /// Values up to 1 pan to the right
    /// Values down to 0 pan to the left
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::Audio;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_panning(0.9);
    /// }
    /// ```
    pub fn set_panning(&self, panning: f32) {
        self.commands
            .write()
            .push_front((AudioCommand::SetPanning(panning), AudioChannel::default()));
    }

    /// Set playback rate for the default channel
    ///
    /// The default value is 1
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::Audio;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_playback_rate(2.0);
    /// }
    /// ```
    pub fn set_playback_rate(&self, playback_rate: f32) {
        self.commands.write().push_front((
            AudioCommand::SetPlaybackRate(playback_rate),
            AudioChannel::default(),
        ));
    }

    /// Play audio in the given channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{Audio, AudioChannel};
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     audio.play_in_channel(asset_server.load("audio.mp3"), &AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn play_in_channel(
        &self,
        audio_source: Handle<AudioSource>,
        channel_id: &AudioChannel,
    ) -> InstanceHandle {
        let instance_handle = InstanceHandle::new();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioCommandArgs {
                settings: PlayAudioSettings {
                    source: audio_source,
                    intro_source: None,
                    looped: false,
                },
                instance_handle: instance_handle.clone(),
            }),
            channel_id.clone(),
        ));

        instance_handle
    }

    /// Play looped audio in the given channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{Audio, AudioChannel};
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     audio.play_looped_in_channel(asset_server.load("audio.mp3"), &AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn play_looped_in_channel(
        &self,
        audio_source: Handle<AudioSource>,
        channel_id: &AudioChannel,
    ) -> InstanceHandle {
        let instance_handle = InstanceHandle::new();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioCommandArgs {
                settings: PlayAudioSettings {
                    source: audio_source,
                    intro_source: None,
                    looped: true,
                },
                instance_handle: instance_handle.clone(),
            }),
            channel_id.clone(),
        ));

        instance_handle
    }

    /// Play looped audio in the given channel with an intro
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{Audio, AudioChannel};
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     audio.play_looped_with_intro_in_channel(
    ///         asset_server.load("intro.mp3"),
    ///         asset_server.load("audio.mp3"),
    ///         &AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn play_looped_with_intro_in_channel(
        &self,
        intro_audio_source: Handle<AudioSource>,
        looped_audio_source: Handle<AudioSource>,
        channel_id: &AudioChannel,
    ) -> InstanceHandle {
        let instance_handle = InstanceHandle::new();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioCommandArgs {
                settings: PlayAudioSettings {
                    source: looped_audio_source,
                    intro_source: Some(intro_audio_source),
                    looped: true,
                },
                instance_handle: instance_handle.clone(),
            }),
            channel_id.clone(),
        ));

        instance_handle
    }

    /// Stop audio in the given channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{Audio, AudioChannel};
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.stop_channel(&AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn stop_channel(&self, channel_id: &AudioChannel) {
        self.commands
            .write()
            .push_front((AudioCommand::Stop, channel_id.clone()));
    }

    /// Pause audio in the given channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{Audio, AudioChannel};
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.pause_channel(&AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn pause_channel(&self, channel_id: &AudioChannel) {
        self.commands
            .write()
            .push_front((AudioCommand::Pause, channel_id.clone()));
    }

    /// Resume audio in the given channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{Audio, AudioChannel};
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.resume_channel(&AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn resume_channel(&self, channel_id: &AudioChannel) {
        self.commands
            .write()
            .push_front((AudioCommand::Resume, channel_id.clone()));
    }

    /// Set the volume for the given channel
    ///
    /// The default value is 1
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{Audio, AudioChannel};
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_volume_in_channel(0.5, &AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn set_volume_in_channel(&self, volume: f32, channel_id: &AudioChannel) {
        self.commands
            .write()
            .push_front((AudioCommand::SetVolume(volume), channel_id.clone()));
    }

    /// Set panning for the given channel
    ///
    /// The default value is 0.5
    /// Values up to 1 pan to the right
    /// Values down to 0 pan to the left
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{Audio, AudioChannel};
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_panning_in_channel(0.9, &AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn set_panning_in_channel(&self, panning: f32, channel_id: &AudioChannel) {
        self.commands
            .write()
            .push_front((AudioCommand::SetPanning(panning), channel_id.clone()));
    }

    /// Set playback rate for the given channel
    ///
    /// The default value is 1
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{Audio, AudioChannel};
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_playback_rate_in_channel(2.0, &AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn set_playback_rate_in_channel(&self, playback_rate: f32, channel_id: &AudioChannel) {
        self.commands.write().push_front((
            AudioCommand::SetPlaybackRate(playback_rate),
            channel_id.clone(),
        ));
    }

    /// Get state for a playback instance.
    pub fn state(&self, instance_handle: InstanceHandle) -> PlaybackState {
        self.states
            .get(&instance_handle)
            .cloned()
            .unwrap_or_else(|| {
                self.commands
                    .read()
                    .iter()
                    .find(|(command, _)| match command {
                        AudioCommand::Play(PlayAudioCommandArgs {
                            instance_handle: handle,
                            settings: _,
                        }) => handle.id == instance_handle.id,
                        _ => false,
                    })
                    .map(|_| PlaybackState::Queued)
                    .unwrap_or(PlaybackState::Stopped)
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bevy::asset::HandleId;

    #[test]
    fn state_is_queued_if_command_is_queued() {
        let audio = Audio::default();
        let audio_handle: Handle<AudioSource> =
            Handle::<AudioSource>::weak(HandleId::default::<AudioSource>());
        let instance_handle = audio.play(audio_handle);

        assert_eq!(audio.state(instance_handle), PlaybackState::Queued);
    }

    #[test]
    fn state_is_stopped_if_command_is_not_queued_and_id_not_in_state_map() {
        let audio = Audio::default();
        let instance_handle = InstanceHandle::new();

        assert_eq!(audio.state(instance_handle), PlaybackState::Stopped);
    }

    #[test]
    fn state_is_fetched_from_state_map() {
        let mut audio = Audio::default();
        let instance_handle = InstanceHandle::new();
        audio.states.insert(
            instance_handle.clone(),
            PlaybackState::Pausing { position: 42. },
        );

        assert_eq!(
            audio.state(instance_handle),
            PlaybackState::Pausing { position: 42. }
        );
    }
}
