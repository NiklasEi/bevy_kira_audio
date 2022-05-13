//! Audio plugin for the game engine Bevy
//!
//! It uses the library Kira to play audio and offers an API to control running game audio
//! via Bevy's ECS.
//!
//! ```edition2018
//! # use bevy_kira_audio::{AudioStreamChannel, Audio, AudioPlugin};
//! # use bevy::prelude::*;
//! # use bevy::asset::AssetPlugin;
//! # use bevy::app::AppExit;
//! fn main() {
//!    let mut app = App::new();
//!    app
//!         .add_plugins(MinimalPlugins)
//!         .add_plugin(AssetPlugin)
//!         .add_plugin(AudioPlugin)
//! #       .add_system(stop)
//!         .add_startup_system(start_background_audio);
//!    app.run();
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

pub use audio::{AudioApp, AudioChannel, InstanceHandle, PlaybackState};
pub use channel::AudioStreamChannel;
pub use source::AudioSource;
pub use stream::{AudioStream, Frame, StreamedAudio};

mod audio;
mod audio_output;
mod channel;
mod source;
mod stream;

use crate::audio_output::{cleanup_stopped_instances, stream_audio_system, AudioOutput};

#[cfg(feature = "flac")]
use crate::source::FlacLoader;
#[cfg(feature = "mp3")]
use crate::source::Mp3Loader;
#[cfg(feature = "ogg")]
use crate::source::OggLoader;
#[cfg(feature = "settings_loader")]
use crate::source::SettingsLoader;
#[cfg(feature = "wav")]
use crate::source::WavLoader;
use bevy::prelude::{
    AddAsset, App, CoreStage, ParallelSystemDescriptorCoercion, Plugin, SystemLabel,
};
use std::marker::PhantomData;

/// A Bevy plugin for audio
///
/// Add this plugin to your Bevy app to get access to
/// the Audio resource
/// ```edition2018
/// # use bevy_kira_audio::{AudioStreamChannel, Audio, AudioPlugin};
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

/// A Bevy plugin for streaming of audio
///
/// This plugin requires [AudioPlugin] to also be active
/// ```edition2018
/// # use bevy_kira_audio::{AudioStream, Frame, StreamedAudio, AudioStreamChannel, Audio, AudioPlugin, AudioStreamPlugin};
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
///         .add_plugin(AudioStreamPlugin::<SineStream>::default())
///         .add_startup_system(start_stream);
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
///
/// # fn stop(mut events: EventWriter<AppExit>) {
/// #     events.send(AppExit)
/// # }
/// ```
#[derive(Default)]
pub struct AudioStreamPlugin<T: AudioStream> {
    _marker: PhantomData<T>,
}

impl<T> Plugin for AudioStreamPlugin<T>
where
    T: AudioStream,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<StreamedAudio<T>>()
            .add_system_to_stage(CoreStage::PostUpdate, stream_audio_system::<T>);
    }
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct ReadmeDoctests;
