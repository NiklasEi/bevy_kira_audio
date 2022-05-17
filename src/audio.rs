use crate::audio_output::{play_audio_channel, update_instance_states};
use crate::source::AudioSource;
use crate::{AudioSystemLabel, ParallelSystemDescriptorCoercion};
use bevy::app::{App, CoreStage};
use bevy::asset::Handle;
use bevy::ecs::system::Resource;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
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
    /// The instance is paused.
    Paused {
        /// Current playback position in seconds
        position: f64,
    },
    /// The instance is fading out and will be paused when the fadeout is finished.
    Pausing {
        /// Current playback position in seconds
        position: f64,
    },
    /// The instance is playing.
    Playing {
        /// Current playback position in seconds
        position: f64,
    },
    /// The instance is queued
    Queued,
    /// The instance cannot be found anymore.
    /// This might might mean it was playing before and is stopped now,
    /// or it never played in the channel were you asked for it.
    Stopped,
    /// The instance is fading out and will be stopped when the fadeout is finished.
    Stopping {
        /// Current playback position in seconds
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

/// Extension trait to add new audio channels to the application
pub trait AudioApp {
    /// Add a new audio channel to the application
    fn add_audio_channel<T: Resource>(&mut self) -> &mut Self;
}

impl AudioApp for App {
    fn add_audio_channel<T: Resource>(&mut self) -> &mut Self {
        self.add_system_to_stage(CoreStage::PostUpdate, play_audio_channel::<T>)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_instance_states::<T>.after(AudioSystemLabel::InstanceCleanup),
            )
            .insert_resource(AudioChannel::<T>::default())
    }
}

/// Channel to play and control audio
///
/// Add your own channels via [`add_audio_channel`](AudioApp::add_audio_channel).
/// By default, there is only the [`AudioChannel<MainTrack>`](crate::Audio) channel.
pub struct AudioChannel<T> {
    pub(crate) commands: RwLock<VecDeque<AudioCommand>>,
    pub(crate) states: HashMap<InstanceHandle, PlaybackState>,
    _marker: PhantomData<T>,
}

impl<T> Default for AudioChannel<T> {
    fn default() -> Self {
        AudioChannel::<T> {
            commands: Default::default(),
            states: Default::default(),
            _marker: PhantomData::default(),
        }
    }
}

impl<T> AudioChannel<T> {
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

        self.commands
            .write()
            .push_front(AudioCommand::Play(PlayAudioCommandArgs {
                settings: PlayAudioSettings {
                    source: audio_source,
                    intro_source: None,
                    looped: false,
                },
                instance_handle: instance_handle.clone(),
            }));

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

        self.commands
            .write()
            .push_front(AudioCommand::Play(PlayAudioCommandArgs {
                settings: PlayAudioSettings {
                    source: audio_source,
                    intro_source: None,
                    looped: true,
                },
                instance_handle: instance_handle.clone(),
            }));

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

        self.commands
            .write()
            .push_front(AudioCommand::Play(PlayAudioCommandArgs {
                settings: PlayAudioSettings {
                    source: looped_audio_source,
                    intro_source: Some(intro_audio_source),
                    looped: true,
                },
                instance_handle: instance_handle.clone(),
            }));

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
        self.commands.write().push_front(AudioCommand::Stop);
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
        self.commands.write().push_front(AudioCommand::Pause);
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
        self.commands.write().push_front(AudioCommand::Resume);
    }

    /// Set the volume for the default channel
    ///
    /// The default value is 1.
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
            .push_front(AudioCommand::SetVolume(volume));
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
            .push_front(AudioCommand::SetPanning(panning));
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
        self.commands
            .write()
            .push_front(AudioCommand::SetPlaybackRate(playback_rate));
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
                    .find(|command| match command {
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
    use crate::Audio;
    use bevy::asset::HandleId;

    #[test]
    fn state_is_queued_if_command_is_queued() {
        let audio = AudioChannel::<Audio>::default();
        let audio_handle: Handle<AudioSource> =
            Handle::<AudioSource>::weak(HandleId::default::<AudioSource>());
        let instance_handle = audio.play(audio_handle);

        assert_eq!(audio.state(instance_handle), PlaybackState::Queued);
    }

    #[test]
    fn state_is_stopped_if_command_is_not_queued_and_id_not_in_state_map() {
        let audio = AudioChannel::<Audio>::default();
        let instance_handle = InstanceHandle::new();

        assert_eq!(audio.state(instance_handle), PlaybackState::Stopped);
    }

    #[test]
    fn state_is_fetched_from_state_map() {
        let mut audio = AudioChannel::<Audio>::default();
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
