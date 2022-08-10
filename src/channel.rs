use crate::audio::{
    AudioCommand, FadeIn, FadeOut, PlayAudioCommand, PlayAudioSettings, TweenCommand,
    TweenCommandKind,
};
use crate::{AudioSource, InstanceHandle, PlaybackState};
use bevy::asset::Handle;
use bevy::utils::HashMap;
use parking_lot::RwLock;
use std::any::TypeId;
use std::collections::VecDeque;
use std::marker::PhantomData;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Channel {
    Typed(TypeId),
    Dynamic(String),
}

/// Play and control audio
pub trait AudioControl {
    /// Play audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     audio.play(asset_server.load("audio.mp3"));
    /// }
    /// ```
    fn play(&self, audio_source: Handle<AudioSource>) -> PlayAudioCommand;

    /// Stop all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.stop();
    /// }
    /// ```
    fn stop(&self) -> TweenCommand<FadeOut>;

    /// Pause all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.pause();
    /// }
    /// ```
    fn pause(&self) -> TweenCommand<FadeOut>;

    /// Resume all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.resume();
    /// }
    /// ```
    fn resume(&self) -> TweenCommand<FadeIn>;

    /// Set the volume
    ///
    /// The default value is 1.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_volume(0.5);
    /// }
    /// ```
    fn set_volume(&self, volume: f64) -> TweenCommand<FadeIn>;

    /// Set panning
    ///
    /// The default value is 0.5
    /// Values up to 1 pan to the right
    /// Values down to 0 pan to the left
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_panning(0.9);
    /// }
    /// ```
    fn set_panning(&self, panning: f64) -> TweenCommand<FadeIn>;

    /// Set playback rate
    ///
    /// The default value is 1
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_playback_rate(2.0);
    /// }
    /// ```
    fn set_playback_rate(&self, playback_rate: f64) -> TweenCommand<FadeIn>;

    /// Get state for a playback instance.
    fn state(&self, instance_handle: InstanceHandle) -> PlaybackState;

    /// Returns `true` if there is any sound in this channel that is in the state `Playing`, `Pausing`, or `Stopping`
    ///
    /// If there are only `Stopped`, `Paused`, or `Queued` sounds, the method will return `false`.
    /// The same result is returned if there are no sounds in the channel at all.
    fn is_playing_sound(&self) -> bool;
}

pub(crate) trait AudioCommandQue {
    fn que(&self, command: AudioCommand);
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

impl<T> AudioCommandQue for AudioChannel<T> {
    fn que(&self, command: AudioCommand) {
        self.commands.write().push_front(command)
    }
}

impl<T> AudioControl for AudioChannel<T> {
    /// Play audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     audio.play(asset_server.load("audio.mp3"));
    /// }
    /// ```
    fn play(&self, audio_source: Handle<AudioSource>) -> PlayAudioCommand {
        PlayAudioCommand::new(InstanceHandle::new(), audio_source, self)
    }

    /// Stop all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.stop();
    /// }
    /// ```
    fn stop(&self) -> TweenCommand<FadeOut> {
        TweenCommand::new(TweenCommandKind::Stop, self)
    }

    /// Pause all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.pause();
    /// }
    /// ```
    fn pause(&self) -> TweenCommand<FadeOut> {
        TweenCommand::new(TweenCommandKind::Pause, self)
    }

    /// Resume all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.resume();
    /// }
    /// ```
    fn resume(&self) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::Resume, self)
    }

    /// Set the volume
    ///
    /// The default value is 1.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_volume(0.5);
    /// }
    /// ```
    fn set_volume(&self, volume: f64) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::SetVolume(volume), self)
    }

    /// Set panning
    ///
    /// The default value is 0.5
    /// Values up to 1 pan to the right
    /// Values down to 0 pan to the left
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_panning(0.9);
    /// }
    /// ```
    fn set_panning(&self, panning: f64) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::SetPanning(panning), self)
    }

    /// Set playback rate
    ///
    /// The default value is 1
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_playback_rate(2.0);
    /// }
    /// ```
    fn set_playback_rate(&self, playback_rate: f64) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::SetPlaybackRate(playback_rate), self)
    }

    /// Get state for a playback instance.
    fn state(&self, instance_handle: InstanceHandle) -> PlaybackState {
        self.states
            .get(&instance_handle)
            .cloned()
            .unwrap_or_else(|| {
                self.commands
                    .read()
                    .iter()
                    .find(|command| match command {
                        AudioCommand::Play(PlayAudioSettings {
                            instance: handle,
                            settings: _,
                            source: _,
                        }) => handle.id == instance_handle.id,
                        _ => false,
                    })
                    .map(|_| PlaybackState::Queued)
                    .unwrap_or(PlaybackState::Stopped)
            })
    }

    /// Returns `true` if there is any sound in this channel that is in the state `Playing`, `Pausing`, or `Stopping`
    ///
    /// If there are only `Stopped`, `Paused`, or `Queued` sounds, the method will return `false`.
    /// The same result is returned if there are no sounds in the channel at all.
    fn is_playing_sound(&self) -> bool {
        self.states
            .iter()
            .fold(false, |playing, (_, state)| match state {
                PlaybackState::Playing { .. }
                | PlaybackState::Pausing { .. }
                | PlaybackState::Stopping { .. } => true,
                _ => playing,
            })
    }
}

/// A dynamic channel to play and control audio
#[derive(Default)]
pub struct DynamicAudioChannel {
    pub(crate) commands: RwLock<VecDeque<AudioCommand>>,
    pub(crate) states: HashMap<InstanceHandle, PlaybackState>,
}

impl AudioCommandQue for DynamicAudioChannel {
    fn que(&self, command: AudioCommand) {
        self.commands.write().push_front(command)
    }
}

impl AudioControl for DynamicAudioChannel {
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     audio.play(asset_server.load("audio.mp3"));
    /// }
    /// ```
    fn play(&self, audio_source: Handle<AudioSource>) -> PlayAudioCommand {
        PlayAudioCommand::new(InstanceHandle::new(), audio_source, self)
    }

    /// Stop all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.stop();
    /// }
    /// ```
    fn stop(&self) -> TweenCommand<FadeOut> {
        TweenCommand::new(TweenCommandKind::Stop, self)
    }

    /// Pause all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.pause();
    /// }
    /// ```
    fn pause(&self) -> TweenCommand<FadeOut> {
        TweenCommand::new(TweenCommandKind::Pause, self)
    }

    /// Resume all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.resume();
    /// }
    /// ```
    fn resume(&self) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::Resume, self)
    }

    /// Set the volume
    ///
    /// The default value is 1.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_volume(0.5);
    /// }
    /// ```
    fn set_volume(&self, volume: f64) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::SetVolume(volume), self)
    }
    /// Set panning
    ///
    /// The default value is 0.5
    /// Values up to 1 pan to the right
    /// Values down to 0 pan to the left
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_panning(0.9);
    /// }
    /// ```
    fn set_panning(&self, panning: f64) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::SetPanning(panning), self)
    }
    /// Set playback rate
    ///
    /// The default value is 1
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_playback_rate(2.0);
    /// }
    /// ```
    fn set_playback_rate(&self, playback_rate: f64) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::SetPlaybackRate(playback_rate), self)
    }

    /// Get state for a playback instance.
    fn state(&self, instance_handle: InstanceHandle) -> PlaybackState {
        self.states
            .get(&instance_handle)
            .cloned()
            .unwrap_or_else(|| {
                self.commands
                    .read()
                    .iter()
                    .find(|command| match command {
                        AudioCommand::Play(PlayAudioSettings {
                            instance: handle,
                            settings: _,
                            source: _,
                        }) => handle.id == instance_handle.id,
                        _ => false,
                    })
                    .map(|_| PlaybackState::Queued)
                    .unwrap_or(PlaybackState::Stopped)
            })
    }

    /// Returns `true` if there is any sound in this channel that is in the state `Playing`, `Pausing`, or `Stopping`
    ///
    /// If there are only `Stopped`, `Paused`, or `Queued` sounds, the method will return `false`.
    /// The same result is returned if there are no sounds in the channel at all.
    fn is_playing_sound(&self) -> bool {
        self.states
            .iter()
            .fold(false, |playing, (_, state)| match state {
                PlaybackState::Playing { .. }
                | PlaybackState::Pausing { .. }
                | PlaybackState::Stopping { .. } => true,
                _ => playing,
            })
    }
}

/// Resource to play and control audio in dynamic channels
///
/// You should only use this if you need a number of audio channels that is not known at compile time.
/// If that is not the case, typed channels are easier to use with Bevy's ECS.
#[derive(Default)]
pub struct DynamicAudioChannels {
    pub(crate) channels: HashMap<String, DynamicAudioChannel>,
}

impl DynamicAudioChannels {
    /// Creates and returns an audio channel for the given key
    ///
    /// If there already is a channel with the given key, it will be stopped and removed.
    pub fn create_channel(&mut self, key: &str) -> &DynamicAudioChannel {
        if self.is_channel(key) {
            self.remove_channel(key);
        }
        self.channels
            .insert(key.to_owned(), DynamicAudioChannel::default());
        self.channels
            .get(key)
            .expect("Failed to retrieve dynamic audio channel")
    }

    /// Remove the channel behind the given key
    ///
    /// All audio in the channel will be stopped before it is removed.
    /// This method will do nothing if there is no channel for the given key.
    pub fn remove_channel(&mut self, key: &str) {
        if let Some(channel) = self.get_channel(key) {
            channel.stop();
        }
        self.channels.remove(key);
    }

    /// Checks if there is a channel available for the given key.
    pub fn is_channel(&self, key: &str) -> bool {
        self.channels.contains_key(key)
    }

    /// Get a channel to play and control audio in
    ///
    /// # Panics
    /// This method will panic if there is no channel for the given key.
    /// If you aren't sure that there is one, you can check with [`is_channel`](Self::is_channel),
    /// or use [`get_channel`](Self::get_channel) instead.
    pub fn channel(&self, key: &str) -> &DynamicAudioChannel {
        assert!(
            self.channels.contains_key(key),
            "Attempting to access dynamic audio channel '{:?}', which doesn't exist.",
            key
        );
        self.channels
            .get(key)
            .expect("Failed to retrieve dynamic audio channel")
    }

    /// Get a channel to play and control audio in
    pub fn get_channel(&self, key: &str) -> Option<&DynamicAudioChannel> {
        assert!(
            self.channels.contains_key(key),
            "Attempting to access dynamic audio channel '{:?}', which doesn't exist.",
            key
        );
        self.channels.get(key)
    }
}

#[cfg(test)]
mod typed_channels {
    use super::*;
    use crate::Audio;
    use bevy::asset::HandleId;

    #[test]
    fn state_is_queued_if_command_is_queued() {
        let audio = AudioChannel::<Audio>::default();
        let audio_handle: Handle<AudioSource> =
            Handle::<AudioSource>::weak(HandleId::default::<AudioSource>());
        let instance_handle = audio.play(audio_handle).handle();

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

    #[test]
    fn finds_playing_sound() {
        let mut audio = AudioChannel::<Audio>::default();
        audio
            .states
            .insert(InstanceHandle::new(), PlaybackState::Queued);
        audio.states.insert(
            InstanceHandle::new(),
            PlaybackState::Paused { position: 42. },
        );
        audio
            .states
            .insert(InstanceHandle::new(), PlaybackState::Stopped);
        assert!(!audio.is_playing_sound());

        audio.states.insert(
            InstanceHandle::new(),
            PlaybackState::Playing { position: 42. },
        );
        assert!(audio.is_playing_sound());
    }
}

#[cfg(test)]
mod dynamic_channels {
    use super::*;
    use bevy::asset::HandleId;

    #[test]
    fn state_is_queued_if_command_is_queued() {
        let mut audio = DynamicAudioChannels::default();
        let audio_handle: Handle<AudioSource> =
            Handle::<AudioSource>::weak(HandleId::default::<AudioSource>());
        let instance_handle = audio.create_channel("test").play(audio_handle).handle();

        assert_eq!(
            audio.channel("test").state(instance_handle),
            PlaybackState::Queued
        );
    }

    #[test]
    fn state_is_stopped_if_command_is_not_queued_and_id_not_in_state_map() {
        let mut audio = DynamicAudioChannels::default();
        let instance_handle = InstanceHandle::new();

        assert_eq!(
            audio.create_channel("test").state(instance_handle),
            PlaybackState::Stopped
        );
    }

    #[test]
    fn state_is_fetched_from_state_map() {
        let mut audio = DynamicAudioChannels::default();
        let instance_handle = InstanceHandle::new();
        audio.create_channel("test");
        audio.channels.get_mut("test").unwrap().states.insert(
            instance_handle.clone(),
            PlaybackState::Pausing { position: 42. },
        );

        assert_eq!(
            audio.channel("test").state(instance_handle),
            PlaybackState::Pausing { position: 42. }
        );
    }

    #[test]
    fn finds_playing_sound() {
        let mut audio = DynamicAudioChannels::default();
        audio.create_channel("test");
        audio
            .channels
            .get_mut("test")
            .unwrap()
            .states
            .insert(InstanceHandle::new(), PlaybackState::Queued);
        audio.channels.get_mut("test").unwrap().states.insert(
            InstanceHandle::new(),
            PlaybackState::Paused { position: 42. },
        );
        audio
            .channels
            .get_mut("test")
            .unwrap()
            .states
            .insert(InstanceHandle::new(), PlaybackState::Stopped);
        assert!(!audio.channel("test").is_playing_sound());

        audio.channels.get_mut("test").unwrap().states.insert(
            InstanceHandle::new(),
            PlaybackState::Playing { position: 42. },
        );
        assert!(audio.channel("test").is_playing_sound());
    }
}
