//! Audio plugin for the game engine Bevy
//!
//! It uses the library Kira to play audio and offers an API to control running game audio
//! via Bevy's ECS.
//!
//! ```
//! use bevy_kira_audio::prelude::*;
//! use bevy::prelude::*;
//! # use bevy::asset::AssetPlugin;
//! # use bevy::app::AppExit;
//!
//! fn main() {
//!    App::new()
//! #       .add_plugins(MinimalPlugins)
//! #       .add_plugins(AssetPlugin::default())
//! # /*
//!         .add_plugins(DefaultPlugins)
//! # */
//!         .add_plugins(AudioPlugin)
//! #       .add_systems(Update, stop)
//!         .add_systems(Startup, start_background_audio)
//!         .run();
//! }
//!
//! fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
//!     audio.play(asset_server.load("background_audio.mp3")).looped();
//! }
//!
//! # fn stop(mut events: EventWriter<AppExit>) {
//! #     events.send(AppExit::Success);
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

mod audio;
mod audio_output;
mod backend_settings;
mod channel;
mod instance;
mod source;
mod spatial;

use std::any::TypeId;
use std::collections::HashMap;

pub use audio::{
    AudioApp, AudioEasing, AudioTween, FadeIn, FadeOut, PlayAudioCommand, PlaybackState,
    TweenCommand,
};
pub use backend_settings::AudioSettings;
use bevy::app::{PostUpdate, PreUpdate};
use bevy::asset::AssetApp;
pub use channel::AudioControl;
use kira::track::TrackHandle;
pub use source::AudioSource;
pub use spatial::EmitterSettings;
pub use spatial::{SpatialAudioEmitter, SpatialAudioPlugin, SpatialAudioReceiver};
/// Most commonly used types
pub mod prelude {
    #[doc(hidden)]
    pub use crate::audio::{
        AudioApp, AudioEasing, AudioTween, FadeIn, FadeOut, PlayAudioCommand, PlaybackState,
        TweenCommand,
    };
    #[doc(hidden)]
    pub use crate::backend_settings::AudioSettings;
    #[doc(hidden)]
    pub use crate::channel::dynamic::{DynamicAudioChannel, DynamicAudioChannels};
    #[doc(hidden)]
    pub use crate::channel::typed::AudioChannel;
    #[doc(hidden)]
    pub use crate::channel::AudioControl;
    #[doc(hidden)]
    pub use crate::instance::{AudioInstance, AudioInstanceAssetsExt};
    #[doc(hidden)]
    #[cfg(feature = "flac")]
    pub use crate::source::flac_loader::*;
    #[doc(hidden)]
    #[cfg(feature = "mp3")]
    pub use crate::source::mp3_loader::*;
    #[doc(hidden)]
    #[cfg(feature = "ogg")]
    pub use crate::source::ogg_loader::*;
    #[doc(hidden)]
    #[cfg(feature = "settings_loader")]
    pub use crate::source::settings_loader::*;
    #[doc(hidden)]
    #[cfg(feature = "wav")]
    pub use crate::source::wav_loader::*;
    #[doc(hidden)]
    pub use crate::source::AudioSource;
    #[doc(hidden)]
    pub use crate::spatial::{SpatialAudioEmitter, SpatialAudioPlugin, SpatialAudioReceiver};
    #[doc(hidden)]
    pub use crate::{Audio, AudioPlugin, MainTrack};
    pub use kira::{
        sound::{
            static_sound::{StaticSoundData, StaticSoundSettings},
            FromFileError, Sound, SoundData,
        },
        Decibels, Frame,
    };
}

use crate::audio_output::{cleanup_stopped_instances, play_dynamic_channels, AudioOutput};

#[cfg(feature = "flac")]
use crate::source::flac_loader::FlacLoader;
#[cfg(feature = "mp3")]
use crate::source::mp3_loader::Mp3Loader;
#[cfg(feature = "ogg")]
use crate::source::ogg_loader::OggLoader;
#[cfg(feature = "settings_loader")]
use crate::source::settings_loader::SettingsLoader;
#[cfg(feature = "wav")]
use crate::source::wav_loader::WavLoader;
use bevy::prelude::{App, IntoScheduleConfigs, Plugin, Resource, SystemSet};
pub use channel::dynamic::DynamicAudioChannel;
pub use channel::dynamic::DynamicAudioChannels;
pub use channel::typed::AudioChannel;
pub use instance::AudioInstance;
pub use instance::AudioInstanceAssetsExt;

/// A Bevy plugin for audio
///
/// Add this plugin to your Bevy app to get access to
/// the Audio resource
/// ```
/// # use bevy_kira_audio::prelude::*;
/// # use bevy::prelude::*;
/// # use bevy::asset::AssetPlugin;
/// # use bevy::app::AppExit;
/// fn main() {
///    let mut app = App::new();
///    app
///         .add_plugins(MinimalPlugins)
///         .add_plugins(AssetPlugin::default())
///         .add_plugins(AudioPlugin)
/// #       .add_systems(Update, stop)
///         .add_systems(Startup, start_background_audio);
///    app.run();
/// }
///
/// fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
///     audio.play(asset_server.load("background_audio.mp3")).looped();
/// }
///
/// # fn stop(mut events: EventWriter<AppExit>) {
/// #     events.send(AppExit::Success);
/// # }
/// ```

#[derive(Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<AudioOutput>()
            .init_asset::<AudioSource>()
            .init_asset::<AudioInstance>();

        #[cfg(feature = "mp3")]
        app.init_asset_loader::<Mp3Loader>();
        #[cfg(feature = "ogg")]
        app.init_asset_loader::<OggLoader>();
        #[cfg(feature = "wav")]
        app.init_asset_loader::<WavLoader>();
        #[cfg(feature = "flac")]
        app.init_asset_loader::<FlacLoader>();

        #[cfg(feature = "settings_loader")]
        app.init_asset_loader::<SettingsLoader>();

        app.init_resource::<DynamicAudioChannels>()
            .init_resource::<TrackRegistry>()
            .add_systems(
                PostUpdate,
                play_dynamic_channels.in_set(AudioSystemSet::PlayDynamicChannels),
            )
            .add_systems(
                PreUpdate,
                cleanup_stopped_instances.in_set(AudioSystemSet::InstanceCleanup),
            )
            .add_audio_channel::<MainTrack>();
    }
}
/// This resource maps a channel type to its live track handle.
#[derive(Resource, Default)]
pub(crate) struct TrackRegistry {
    handles: HashMap<TypeId, TrackHandle>,
}
/// Labels for audio systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum AudioSystemSet {
    /// Label for systems in [`CoreStage::PreUpdate`] that clean up tracked audio instances
    InstanceCleanup,
    /// Label for system in [`CoreStage::PostUpdate`] that processes audio commands for dynamic channels
    PlayDynamicChannels,
    /// Label for systems in [`CoreStage::PostUpdate`] that process audio commands for typed channels
    PlayTypedChannels,
}

/// The default audio channel
///
/// Alias for the [`AudioChannel<MainTrack>`] resource. Use it to play and control sound on the main track.
/// You can add your own channels via [`add_audio_channel`](AudioApp::add_audio_channel).
pub type Audio = AudioChannel<MainTrack>;

/// Type for the default audio channel
///
/// Use it via the [`AudioChannel<MainTrack>`] resource to play and control sound on the main track.
/// You can add your own channels via [`add_audio_channel`](AudioApp::add_audio_channel).
///
/// You can use [`Audio`] as a type alias for [`AudioChannel<MainTrack>`]
#[derive(Resource)]
pub struct MainTrack;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct ReadmeDoctests;
