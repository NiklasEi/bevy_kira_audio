use bevy::prelude::*;

pub use audio::Audio;
pub use source::AudioSource;

mod audio;
mod audio_output;
mod source;

use crate::audio_output::{play_queued_audio_system, AudioOutput};

#[cfg(feature = "flac")]
use crate::source::FlacLoader;
#[cfg(feature = "mp3")]
use crate::source::Mp3Loader;
#[cfg(feature = "vorbis")]
use crate::source::OggLoader;
#[cfg(feature = "wav")]
use crate::source::WavLoader;

#[derive(Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_thread_local_resource::<AudioOutput>()
            .add_asset::<AudioSource>();

        #[cfg(feature = "mp3")]
        app.init_asset_loader::<Mp3Loader>();
        #[cfg(feature = "vorbis")]
        app.init_asset_loader::<OggLoader>();
        #[cfg(feature = "wav")]
        app.init_asset_loader::<WavLoader>();
        #[cfg(feature = "flac")]
        app.init_asset_loader::<FlacLoader>();

        app.init_resource::<Audio>()
            .add_system_to_stage(stage::POST_UPDATE, play_queued_audio_system.system());
    }
}
