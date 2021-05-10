use crate::channel::AudioChannel;
use crate::source::AudioSource;
use bevy::prelude::Handle;
use parking_lot::RwLock;
use std::collections::VecDeque;

pub enum AudioCommands {
    Play(PlayAudioSettings),
    SetVolume(f32),
    SetPanning(f32),
    SetPlaybackRate(f32),
    Stop,
    Pause,
    Resume,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct PlayAudioSettings {
    pub source: Handle<AudioSource>,
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
    pub(crate) commands: RwLock<VecDeque<(AudioCommands, AudioChannel)>>,
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
    pub fn play(&self, audio_source: Handle<AudioSource>) {
        self.commands.write().push_front((
            AudioCommands::Play(PlayAudioSettings {
                source: audio_source,
                looped: false,
            }),
            AudioChannel::default(),
        ));
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
    pub fn play_looped(&self, audio_source: Handle<AudioSource>) {
        self.commands.write().push_front((
            AudioCommands::Play(PlayAudioSettings {
                source: audio_source,
                looped: true,
            }),
            AudioChannel::default(),
        ));
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
            .push_front((AudioCommands::Stop, AudioChannel::default()));
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
            .push_front((AudioCommands::Pause, AudioChannel::default()));
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
            .push_front((AudioCommands::Resume, AudioChannel::default()));
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
            .push_front((AudioCommands::SetVolume(volume), AudioChannel::default()));
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
            .push_front((AudioCommands::SetPanning(panning), AudioChannel::default()));
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
            AudioCommands::SetPlaybackRate(playback_rate),
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
    pub fn play_in_channel(&self, audio_source: Handle<AudioSource>, channel_id: &AudioChannel) {
        self.commands.write().push_front((
            AudioCommands::Play(PlayAudioSettings {
                source: audio_source,
                looped: false,
            }),
            channel_id.clone(),
        ));
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
    ) {
        self.commands.write().push_front((
            AudioCommands::Play(PlayAudioSettings {
                source: audio_source,
                looped: true,
            }),
            channel_id.clone(),
        ));
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
            .push_front((AudioCommands::Stop, channel_id.clone()));
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
            .push_front((AudioCommands::Pause, channel_id.clone()));
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
            .push_front((AudioCommands::Resume, channel_id.clone()));
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
            .push_front((AudioCommands::SetVolume(volume), channel_id.clone()));
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
            .push_front((AudioCommands::SetPanning(panning), channel_id.clone()));
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
            AudioCommands::SetPlaybackRate(playback_rate),
            channel_id.clone(),
        ));
    }
}
