//! Common audio types

use crate::audio_output::{play_audio_channel, update_instance_states};
use crate::channel::{AudioChannel, AudioCommandQue, AudioInstance};
use crate::source::AudioSource;
use crate::{AudioSystemLabel, ParallelSystemDescriptorCoercion};
use bevy::app::{App, CoreStage};
use bevy::asset::{Handle, HandleId};
use bevy::ecs::system::Resource;
use bevy::prelude::default;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::LoopBehavior;
use std::marker::PhantomData;
use std::time::Duration;

pub(crate) enum AudioCommand {
    Play(PlayAudioSettings),
    SetVolume(f64, Option<AudioTween>),
    SetPanning(f64, Option<AudioTween>),
    SetPlaybackRate(f64, Option<AudioTween>),
    Stop(Option<AudioTween>),
    Pause(Option<AudioTween>),
    Resume(Option<AudioTween>),
}

#[derive(Clone, Default)]
pub(crate) struct PartialSoundSettings {
    pub(crate) loop_behavior: Option<Option<f64>>,
    pub(crate) volume: Option<f64>,
    pub(crate) playback_rate: Option<f64>,
    pub(crate) start_position: Option<f64>,
    pub(crate) panning: Option<f64>,
    pub(crate) reverse: Option<bool>,
    pub(crate) fade_in: Option<AudioTween>,
}

/// Different kinds of easing for fade-in and fade-out
pub type AudioEasing = kira::tween::Easing;

#[derive(Clone)]
/// A tween for audio transitions
///
/// Use the default for almost instantaneous transitions without audio artifacts
pub struct AudioTween {
    duration: Duration,
    easing: AudioEasing,
}

impl AudioTween {
    /// Create a new tween with the given duration and easing
    pub fn new(duration: Duration, easing: AudioEasing) -> Self {
        AudioTween { duration, easing }
    }

    /// Create a new linear tween with the given duration
    pub fn linear(duration: Duration) -> Self {
        AudioTween {
            duration,
            easing: AudioEasing::Linear,
        }
    }

    /// Set an easing for the tween
    pub fn with_easing(mut self, easing: AudioEasing) -> Self {
        self.easing = easing;

        self
    }
}

impl Default for AudioTween {
    fn default() -> Self {
        AudioTween::new(Duration::from_millis(10), AudioEasing::Linear)
    }
}

pub fn map_tween(tween: &Option<AudioTween>) -> kira::tween::Tween {
    match tween {
        Some(tween) => tween.into(),
        None => kira::tween::Tween::default(),
    }
}

impl From<AudioTween> for kira::tween::Tween {
    fn from(tween: AudioTween) -> Self {
        (&tween).into()
    }
}

impl From<&AudioTween> for kira::tween::Tween {
    fn from(tween: &AudioTween) -> Self {
        kira::tween::Tween {
            duration: tween.duration,
            easing: tween.easing,
            ..default()
        }
    }
}

impl PartialSoundSettings {
    pub(crate) fn apply(&self, sound: &mut StaticSoundData) {
        if let Some(loop_behavior) = self.loop_behavior {
            if let Some(start_position) = loop_behavior {
                sound.settings.loop_behavior = Some(LoopBehavior { start_position })
            } else {
                sound.settings.loop_behavior = None;
            }
        }
        if let Some(volume) = self.volume {
            sound.settings.volume = volume.into();
        }
        if let Some(playback_rate) = self.playback_rate {
            sound.settings.playback_rate = playback_rate.into();
        }
        if let Some(start_position) = self.start_position {
            sound.settings.start_position = start_position;
        }
        if let Some(panning) = self.panning {
            sound.settings.panning = panning;
        }
        if let Some(reverse) = self.reverse {
            sound.settings.reverse = reverse;
        }
        if let Some(AudioTween { duration, easing }) = self.fade_in {
            sound.settings.fade_in_tween = Some(kira::tween::Tween {
                duration,
                easing,
                ..default()
            });
        }
    }
}

pub struct PlayAudioSettings {
    pub(crate) instance_handle: Handle<AudioInstance>,
    pub(crate) source: Handle<AudioSource>,
    pub(crate) settings: PartialSoundSettings,
}

impl<'a> From<&mut PlayAudioCommand<'a>> for PlayAudioSettings {
    fn from(command: &mut PlayAudioCommand<'a>) -> Self {
        PlayAudioSettings {
            instance_handle: command.instance_handle.clone(),
            source: command.source.clone(),
            settings: command.settings.clone(),
        }
    }
}

pub struct PlayAudioCommand<'a> {
    pub(crate) instance_handle: Handle<AudioInstance>,
    pub(crate) source: Handle<AudioSource>,
    pub(crate) settings: PartialSoundSettings,
    pub(crate) que: &'a dyn AudioCommandQue,
}

impl<'a> Drop for PlayAudioCommand<'a> {
    fn drop(&mut self) {
        self.que.que(AudioCommand::Play(self.into()));
    }
}

impl<'a> PlayAudioCommand<'a> {
    pub(crate) fn new(source: Handle<AudioSource>, que: &'a dyn AudioCommandQue) -> Self {
        let handle_id = HandleId::random::<AudioInstance>();
        Self {
            instance_handle: Handle::<AudioInstance>::weak(handle_id),
            source,
            settings: PartialSoundSettings::default(),
            que,
        }
    }

    pub fn looped(&mut self) -> &mut Self {
        self.settings.loop_behavior = Some(Some(0.0));

        self
    }

    pub fn loop_from(&mut self, loop_start_position: f64) -> &mut Self {
        self.settings.loop_behavior = Some(Some(loop_start_position));

        self
    }

    pub fn with_volume(&mut self, volume: f64) -> &mut Self {
        self.settings.volume = Some(volume);

        self
    }

    pub fn with_playback_rate(&mut self, playback_rate: f64) -> &mut Self {
        self.settings.playback_rate = Some(playback_rate);

        self
    }

    pub fn start_from(&mut self, start_position: f64) -> &mut Self {
        self.settings.start_position = Some(start_position);

        self
    }

    pub fn with_panning(&mut self, panning: f64) -> &mut Self {
        self.settings.panning = Some(panning);

        self
    }

    pub fn reverse(&mut self) -> &mut Self {
        let current = self.settings.reverse.unwrap_or(false);
        self.settings.reverse = Some(!current);

        self
    }

    pub fn linear_fade_in(&mut self, duration: Duration) -> &mut Self {
        self.settings.fade_in = Some(AudioTween::linear(duration));

        self
    }

    pub fn fade_in(&mut self, tween: AudioTween) -> &mut Self {
        self.settings.fade_in = Some(tween);

        self
    }

    pub fn handle(&mut self) -> Handle<AudioInstance> {
        self.instance_handle.clone()
    }
}

pub(crate) enum TweenCommandKind {
    SetVolume(f64),
    SetPanning(f64),
    SetPlaybackRate(f64),
    Stop,
    Pause,
    Resume,
}

impl TweenCommandKind {
    fn to_command(&self, tween: Option<AudioTween>) -> AudioCommand {
        match self {
            TweenCommandKind::SetVolume(volume) => AudioCommand::SetVolume(*volume, tween),
            TweenCommandKind::SetPanning(panning) => AudioCommand::SetPanning(*panning, tween),
            TweenCommandKind::SetPlaybackRate(playback_rate) => {
                AudioCommand::SetPlaybackRate(*playback_rate, tween)
            }
            TweenCommandKind::Stop => AudioCommand::Stop(tween),
            TweenCommandKind::Pause => AudioCommand::Pause(tween),
            TweenCommandKind::Resume => AudioCommand::Resume(tween),
        }
    }
}

pub struct FadeIn;
pub struct FadeOut;

pub struct TweenCommand<'a, Fade> {
    pub(crate) kind: TweenCommandKind,
    pub(crate) tween: Option<AudioTween>,
    pub(crate) que: &'a dyn AudioCommandQue,
    _marker: PhantomData<Fade>,
}

impl<'a, Fade> Drop for TweenCommand<'a, Fade> {
    fn drop(&mut self) {
        self.que.que(self.kind.to_command(self.tween.take()));
    }
}

impl<'a, Fade> TweenCommand<'a, Fade> {
    pub(crate) fn new(kind: TweenCommandKind, que: &'a dyn AudioCommandQue) -> Self {
        Self {
            kind,
            tween: None,
            que,
            _marker: PhantomData::<Fade>::default(),
        }
    }
}

impl<'a> TweenCommand<'a, FadeIn> {
    pub fn linear_fade_in(&mut self, duration: Duration) -> &mut Self {
        self.tween = Some(AudioTween::linear(duration));

        self
    }

    pub fn fade_in(&mut self, tween: AudioTween) -> &mut Self {
        self.tween = Some(tween);

        self
    }
}

impl<'a> TweenCommand<'a, FadeOut> {
    pub fn linear_fade_out(&mut self, duration: Duration) -> &mut Self {
        self.tween = Some(AudioTween::linear(duration));

        self
    }

    pub fn fade_out(&mut self, tween: AudioTween) -> &mut Self {
        self.tween = Some(tween);

        self
    }
}

pub enum AudioCommandResult {
    Ok,
    Retry,
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

impl From<StaticSoundHandle> for PlaybackState {
    fn from(sound_handle: StaticSoundHandle) -> Self {
        (&sound_handle).into()
    }
}

impl From<&StaticSoundHandle> for PlaybackState {
    fn from(sound_handle: &StaticSoundHandle) -> Self {
        let position = sound_handle.position();
        match sound_handle.state() {
            kira::sound::static_sound::PlaybackState::Playing => {
                PlaybackState::Playing { position }
            }
            kira::sound::static_sound::PlaybackState::Paused => PlaybackState::Paused { position },
            kira::sound::static_sound::PlaybackState::Stopped => PlaybackState::Stopped,
            kira::sound::static_sound::PlaybackState::Pausing => {
                PlaybackState::Pausing { position }
            }
            kira::sound::static_sound::PlaybackState::Stopping => {
                PlaybackState::Stopping { position }
            }
        }
    }
}

/// Extension trait to add new audio channels to the application
pub trait AudioApp {
    /// Add a new audio channel to the application
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_kira_audio::prelude::*;
    ///
    /// fn main() {
    ///     App::new()
    ///         .add_plugins(DefaultPlugins)
    ///         .add_plugin(AudioPlugin)
    ///         .add_audio_channel::<Background>()
    ///         .add_startup_system(play)
    ///         .run();
    /// }
    ///
    /// fn play(background: Res<AudioChannel<Background>>, asset_server: Res<AssetServer>) {
    ///     background.play(asset_server.load("sounds/loop.ogg"));
    /// }
    ///
    /// struct Background;
    /// ```
    fn add_audio_channel<T: Resource>(&mut self) -> &mut Self;
}

impl AudioApp for App {
    fn add_audio_channel<T: Resource>(&mut self) -> &mut Self {
        self.add_system_to_stage(
            CoreStage::PostUpdate,
            play_audio_channel::<T>.label(AudioSystemLabel::PlayTypedChannels),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            update_instance_states::<T>.after(AudioSystemLabel::InstanceCleanup),
        )
        .insert_resource(AudioChannel::<T>::default())
    }
}
