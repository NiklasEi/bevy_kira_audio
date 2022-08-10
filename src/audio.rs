//! Common audio types

use crate::audio_output::{play_audio_channel, update_instance_states, InstanceState};
use crate::channel::AudioChannel;
use crate::source::AudioSource;
use crate::{AudioSystemLabel, ParallelSystemDescriptorCoercion};
use bevy::app::{App, CoreStage};
use bevy::asset::Handle;
use bevy::ecs::system::Resource;
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
