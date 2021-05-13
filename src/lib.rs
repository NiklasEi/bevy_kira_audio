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
#![warn(unused_imports, missing_docs)]

pub use audio::Audio;
pub use channel::AudioChannel;
pub use source::AudioSource;
pub use stream::{AudioStream, Frame, StreamedAudio};

mod audio;
mod audio_output;
mod channel;
mod source;
mod stream;

use crate::audio_output::{play_queued_audio_system, stream_audio_system, AudioOutput};

#[cfg(feature = "flac")]
use crate::source::FlacLoader;
#[cfg(feature = "mp3")]
use crate::source::Mp3Loader;
#[cfg(feature = "ogg")]
use crate::source::OggLoader;
#[cfg(feature = "wav")]
use crate::source::WavLoader;
use bevy::prelude::{AddAsset, AppBuilder, CoreStage, IntoExclusiveSystem, Plugin};
use std::marker::PhantomData;

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

        app.init_resource::<Audio>().add_system_to_stage(
            CoreStage::PostUpdate,
            play_queued_audio_system.exclusive_system(),
        );
    }
}

/// A Bevy plugin for streaming of audio
///
/// This plugin requires [AudioPlugin] to also be active
/// ```edition2018
/// # use bevy_kira_audio::{AudioStream, Frame, StreamedAudio, AudioChannel, Audio, AudioPlugin, AudioStreamPlugin};
/// # use bevy::prelude::*;
/// fn main() {
///    let mut app = App::build();
///    app
///         .add_plugins(DefaultPlugins)
///         .add_plugin(AudioPlugin)
///         .add_plugin(AudioStreamPlugin::<SineStream>::default())
///         .add_startup_system(start_stream.system());
///    app.run();
/// }
///
/// #[derive(Debug, Default)]
/// struct SineStream {
///     t: f64,
///     note: f64,
///     frequency: f64
/// }
///
/// impl AudioStream for SineStream {
///     fn next(&mut self, _: f64) -> Frame {
///         let increment = 2.0 * std::f64::consts::PI * self.note / self.frequency;
///         self.t += increment;
///
///         let sample: f64 = self.t.sin();
///         Frame {
///             left: sample as f32,
///             right: sample as f32,
///         }
///     }
/// }
///
///fn start_stream(audio: Res<StreamedAudio<SineStream>>) {
///    audio.stream(SineStream {
///        t: 0.0,
///        note: 440.0,
///        frequency: 44_000.0,
///    });
///}
/// ```
#[derive(Default)]
pub struct AudioStreamPlugin<T: AudioStream> {
    _marker: PhantomData<T>,
}

impl<T> Plugin for AudioStreamPlugin<T>
where
    T: AudioStream,
{
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<StreamedAudio<T>>().add_system_to_stage(
            CoreStage::PostUpdate,
            stream_audio_system::<T>.exclusive_system(),
        );
    }
}
