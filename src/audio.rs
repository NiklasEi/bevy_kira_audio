use crate::channel::AudioChannel;
use crate::source::AudioSource;
use atomic::Atomic;
use bevy::prelude::Handle;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub(crate) enum AudioCommand {
    Play(PlayAudioSettings, InstanceHandlePriv),
    SetVolume(f32),
    SetPanning(f32),
    SetPlaybackRate(f32),
    Stop,
    Pause,
    Resume,
}

pub enum AudioCommandResult {
    Ok,
    Retry,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct PlayAudioSettings {
    pub source: Handle<AudioSource>,
    pub intro_source: Option<Handle<AudioSource>>,
    pub looped: bool,
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
}

pub struct InstanceHandle {
    position: Arc<Atomic<f64>>,
}

#[derive(Clone)]
pub(crate) struct InstanceHandlePriv {
    pub(crate) position: Arc<Atomic<f64>>,
}

pub const INSTANCE_HANDLE_POSITION_INVALID: f64 = -1.;

impl InstanceHandle {
    fn new_pair() -> (InstanceHandle, InstanceHandlePriv) {
        let pos = Arc::new(Atomic::new(INSTANCE_HANDLE_POSITION_INVALID));
        (
            InstanceHandle {
                position: pos.clone(),
            },
            InstanceHandlePriv {
                position: pos,
            }
            )
    }

    pub fn position(&self) -> f64 {
        self.position.load(Ordering::Acquire)
    }
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
        let (instance, instance_priv) = InstanceHandle::new_pair();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioSettings {
                source: audio_source,
                intro_source: None,
                looped: false,
            }, instance_priv),
            AudioChannel::default(),
        ));

        instance
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
        let (instance, instance_priv) = InstanceHandle::new_pair();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioSettings {
                source: audio_source,
                intro_source: None,
                looped: true,
            }, instance_priv),
            AudioChannel::default(),
        ));

        instance
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
        let (instance, instance_priv) = InstanceHandle::new_pair();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioSettings {
                source: looped_audio_source,
                intro_source: Some(intro_audio_source),
                looped: true,
            }, instance_priv),
            AudioChannel::default(),
        ));

        instance
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
        channel_id: &AudioChannel
    ) -> InstanceHandle {
        let (instance, instance_priv) = InstanceHandle::new_pair();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioSettings {
                source: audio_source,
                intro_source: None,
                looped: false,
            }, instance_priv),
            channel_id.clone(),
        ));

        instance
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
        let (instance, instance_priv) = InstanceHandle::new_pair();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioSettings {
                source: audio_source,
                intro_source: None,
                looped: true,
            }, instance_priv),
            channel_id.clone(),
        ));

        instance
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
        let (instance, instance_priv) = InstanceHandle::new_pair();

        self.commands.write().push_front((
            AudioCommand::Play(PlayAudioSettings {
                source: looped_audio_source,
                intro_source: Some(intro_audio_source),
                looped: true,
            }, instance_priv),
            channel_id.clone(),
        ));

        instance
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
}
