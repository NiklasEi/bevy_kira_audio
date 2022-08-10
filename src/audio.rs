//! Common audio types

use crate::audio_output::{play_audio_channel, update_instance_states, InstanceState};
use crate::channel::{AudioChannel, AudioCommandQue};
use crate::source::AudioSource;
use crate::{AudioSystemLabel, ParallelSystemDescriptorCoercion};
use bevy::app::{App, CoreStage};
use bevy::asset::Handle;
use bevy::ecs::system::Resource;
use bevy::prelude::default;
use kira::sound::static_sound::StaticSoundData;
use kira::LoopBehavior;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

pub(crate) enum AudioCommand {
    Play(PlayAudioSettings),
    SetVolume(f64, Option<Tween>),
    SetPanning(f64, Option<Tween>),
    SetPlaybackRate(f64, Option<Tween>),
    Stop(Option<Tween>),
    Pause(Option<Tween>),
    Resume(Option<Tween>),
}

#[derive(Clone, Default)]
pub(crate) struct PartialSoundSettings {
    pub(crate) loop_behavior: Option<Option<f64>>,
    pub(crate) volume: Option<f64>,
    pub(crate) playback_rate: Option<f64>,
    pub(crate) start_position: Option<f64>,
    pub(crate) panning: Option<f64>,
    pub(crate) reverse: Option<bool>,
    pub(crate) fade_in: Option<Tween>,
}

/// Different kinds of easing for fade-in and fade-out
pub type Easing = kira::tween::Easing;

#[derive(Clone)]
pub struct Tween(Duration, Easing);

pub fn map_tween(tween: &Option<Tween>) -> kira::tween::Tween {
    match tween {
        Some(tween) => kira::tween::Tween {
            duration: tween.0,
            easing: tween.1,
            ..default()
        },
        None => kira::tween::Tween::default(),
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
        if let Some(Tween(duration, easing)) = self.fade_in {
            sound.settings.fade_in_tween = Some(kira::tween::Tween {
                duration,
                easing,
                ..default()
            });
        }
    }
}

pub struct PlayAudioSettings {
    pub(crate) instance: InstanceHandle,
    pub(crate) source: Handle<AudioSource>,
    pub(crate) settings: PartialSoundSettings,
}

impl<'a> From<&mut PlayAudioCommand<'a>> for PlayAudioSettings {
    fn from(command: &mut PlayAudioCommand<'a>) -> Self {
        PlayAudioSettings {
            instance: command.instance.clone(),
            source: command.source.clone(),
            settings: command.settings.clone(),
        }
    }
}

pub struct PlayAudioCommand<'a> {
    pub(crate) instance: InstanceHandle,
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
    pub(crate) fn new(
        instance: InstanceHandle,
        source: Handle<AudioSource>,
        que: &'a dyn AudioCommandQue,
    ) -> Self {
        Self {
            instance,
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
        self.settings.fade_in = Some(Tween(duration, Easing::Linear));

        self
    }

    pub fn fade_in(&mut self, duration: Duration, easing: Easing) -> &mut Self {
        self.settings.fade_in = Some(Tween(duration, easing));

        self
    }

    #[must_use]
    pub fn handle(&mut self) -> InstanceHandle {
        self.instance.clone()
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
    fn to_command(&self, tween: Option<Tween>) -> AudioCommand {
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
    pub(crate) tween: Option<Tween>,
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
        self.tween = Some(Tween(duration, Easing::Linear));

        self
    }

    pub fn fade_in(&mut self, duration: Duration, easing: Easing) -> &mut Self {
        self.tween = Some(Tween(duration, easing));

        self
    }
}

impl<'a> TweenCommand<'a, FadeOut> {
    pub fn linear_fade_out(&mut self, duration: Duration) -> &mut Self {
        self.tween = Some(Tween(duration, Easing::Linear));

        self
    }

    pub fn fade_out(&mut self, duration: Duration, easing: Easing) -> &mut Self {
        self.tween = Some(Tween(duration, easing));

        self
    }
}

pub enum AudioCommandResult {
    Ok,
    Retry,
}

/// Allows you to interact with a playing sound.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InstanceHandle {
    pub(crate) id: u64,
}

static NEXT_INSTANCE_HANDLE_ID: AtomicU64 = AtomicU64::new(0);

impl InstanceHandle {
    pub(crate) fn new() -> InstanceHandle {
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

impl From<&InstanceState> for PlaybackState {
    fn from(state: &InstanceState) -> Self {
        let position = state.kira.position();
        match state.kira.state() {
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
