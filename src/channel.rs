pub mod dynamic;
pub mod typed;

use crate::audio::{AudioCommand, FadeIn, FadeOut, PlayAudioCommand, TweenCommand};
use crate::instance::AudioInstance;
use crate::{AudioSource, PlaybackState};
use bevy::asset::Handle;
use kira::sound::static_sound::StaticSoundData;
use kira::{Decibels, Panning, Value};
use std::any::TypeId;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Channel {
    Typed(TypeId),
    Dynamic(String),
}

pub(crate) struct ChannelState {
    pub(crate) paused: bool,
    pub(crate) volume: Decibels,
    pub(crate) playback_rate: f64,
    pub(crate) panning: Panning,
}

impl Default for ChannelState {
    fn default() -> Self {
        ChannelState {
            paused: false,
            volume: 1.0.into(),
            playback_rate: 1.0,
            panning: Panning(0.5),
        }
    }
}

impl ChannelState {
    pub(crate) fn apply(&self, sound: &mut StaticSoundData) {
        sound.settings.volume = Value::Fixed(self.volume);
        sound.settings.playback_rate = self.playback_rate.into();
        sound.settings.panning = Value::Fixed(self.panning);
    }
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
    /// This method supports setting the volume in Decibels or as Amplitude.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_volume(0.5);
    /// }
    /// ```
    fn set_volume(&self, volume: impl Into<Decibels>) -> TweenCommand<FadeIn>;

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
    ///     audio.set_panning(kira::Panning(0.9));
    /// }
    /// ```
    fn set_panning(&self, panning: Panning) -> TweenCommand<FadeIn>;

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
    fn state(&self, instance_handle: &Handle<AudioInstance>) -> PlaybackState;

    /// Returns `true` if there is any sound in this channel that is in the state `Playing`, `Pausing`, or `Stopping`
    ///
    /// If there are only `Stopped`, `Paused`, or `Queued` sounds, the method will return `false`.
    /// The same result is returned if there are no sounds in the channel at all.
    fn is_playing_sound(&self) -> bool;
}

pub(crate) trait AudioCommandQue {
    fn que(&self, command: AudioCommand);
}
