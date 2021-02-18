//! # Bevy Kira audio
//!
//! This crate is an audio plugin for the game engine Bevy. It uses the library
//! Kira to play audio and offers an API to control running audio.
//!
//! See the repository <https://github.com/NiklasEi/bevy_kira_audio/> for additional
//! documentation and usage examples.
//! ```edition2018
//! # use bevy_kira_audio::{AudioChannel, Audio, AudioPlugin};
//! # use bevy::prelude::*;
//! fn main() {
//!    let mut app = App::build();
//!    app
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(AudioPlugin)
//!         .add_startup_system(start_background_audio.system());
//!    app.run();
//! }
//!
//! fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
//!     audio.play_looped(asset_server.load("background_audio.mp3"));
//! }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs, unused_imports)]

use bevy::prelude::*;

pub use audio::Audio;
pub use source::AudioSource;

mod audio;
mod audio_output;
mod channel;
mod source;

use crate::audio_output::{play_queued_audio_system, AudioOutput};

pub use channel::AudioChannel;

#[cfg(feature = "flac")]
use crate::source::FlacLoader;
#[cfg(feature = "mp3")]
use crate::source::Mp3Loader;
#[cfg(feature = "ogg")]
use crate::source::OggLoader;
#[cfg(feature = "wav")]
use crate::source::WavLoader;

/// A Bevy plugin to add audio functionallity
///
/// Add this plugin to your Bevy app to get access to
/// the Audio resource
/// ```edition2018
/// # use bevy_kira_audio::{AudioChannel, Audio, AudioPlugin};
/// # use bevy::prelude::*;
/// fn main() {
///    let mut app = App::build();
///    app
///         .add_plugins(DefaultPlugins)
///         .add_plugin(AudioPlugin)
///         .add_startup_system(start_background_audio.system());
///    app.run();
/// }
///
/// fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
///     audio.play_looped(asset_server.load("background_audio.mp3"));
/// }
/// ```
#[derive(Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut AppBuilder) {
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

        app.init_resource::<Audio>()
            .add_system_to_stage(stage::POST_UPDATE, play_queued_audio_system.exclusive_system());
    }
}
