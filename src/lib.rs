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
//! #       .add_plugin(AssetPlugin)
//! # /*
//!         .add_plugins(DefaultPlugins)
//! # */
//!         .add_plugin(AudioPlugin)
//! #       .add_system(stop)
//!         .add_startup_system(start_background_audio)
//!         .run();
//! }
//!
//! fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
//!     audio.play(asset_server.load("background_audio.mp3")).looped();
//! }
//!
//! # fn stop(mut events: EventWriter<AppExit>) {
//! #     events.send(AppExit)
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

pub use audio::{AudioApp, AudioEasing, AudioTween, PlaybackState};
pub use backend_settings::AudioSettings;
pub use channel::AudioControl;
pub use source::AudioSource;

/// Most commonly used types
pub mod prelude {
    #[doc(hidden)]
    pub use crate::audio::{AudioApp, AudioEasing, AudioTween, PlaybackState};
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
    pub use crate::source::AudioSource;
    #[doc(hidden)]
    pub use crate::{Audio, AudioPlugin, MainTrack};
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
use bevy::prelude::{
    AddAsset, App, CoreStage, ParallelSystemDescriptorCoercion, Plugin, SystemLabel,
};
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
///         .add_plugin(AssetPlugin)
///         .add_plugin(AudioPlugin)
/// #       .add_system(stop)
///         .add_startup_system(start_background_audio);
///    app.run();
/// }
///
/// fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
///     audio.play(asset_server.load("background_audio.mp3")).looped();
/// }
///
/// # fn stop(mut events: EventWriter<AppExit>) {
/// #     events.send(AppExit)
/// # }
/// ```
#[derive(Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<AudioOutput>()
            .add_asset::<AudioSource>()
            .add_asset::<AudioInstance>();

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
            .add_system_to_stage(
                CoreStage::PostUpdate,
                play_dynamic_channels.label(AudioSystemLabel::PlayDynamicChannels),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                cleanup_stopped_instances.label(AudioSystemLabel::InstanceCleanup),
            )
            .add_audio_channel::<MainTrack>();
    }
}

/// Labels for audio systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum AudioSystemLabel {
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
/// You can add your own channels via [`add_audio_channel`](audio::AudioApp::add_audio_channel).
pub type Audio = AudioChannel<MainTrack>;

/// Type for the default audio channel
///
/// Use it via the [`AudioChannel<MainTrack>`] resource to play and control sound on the main track.
/// You can add your own channels via [`add_audio_channel`](audio::AudioApp::add_audio_channel).
///
/// You can use [`Audio`] as a type alias for [`AudioChannel<MainTrack>`]
pub struct MainTrack;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct ReadmeDoctests;
