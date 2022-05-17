//! Audio plugin for the game engine Bevy
//!
//! It uses the library Kira to play audio and offers an API to control running game audio
//! via Bevy's ECS.
//!
//! ```
//! use bevy_kira_audio::{Audio, AudioPlugin};
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
//!     audio.play_looped(asset_server.load("background_audio.mp3"));
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
mod settings;
mod source;

pub use audio::{AudioApp, AudioChannel, InstanceHandle, PlaybackState};
pub use settings::AudioSettings;
pub use source::AudioSource;

use crate::audio_output::{cleanup_stopped_instances, AudioOutput};

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

/// A Bevy plugin for audio
///
/// Add this plugin to your Bevy app to get access to
/// the Audio resource
/// ```edition2018
/// # use bevy_kira_audio::{Audio, AudioPlugin};
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
///     audio.play_looped(asset_server.load("background_audio.mp3"));
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
            .add_asset::<AudioSource>();

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

        app.add_system_to_stage(
            CoreStage::PreUpdate,
            cleanup_stopped_instances.label(AudioSystemLabel::InstanceCleanup),
        )
        .add_audio_channel::<MainTrack>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub(crate) enum AudioSystemLabel {
    InstanceCleanup,
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
