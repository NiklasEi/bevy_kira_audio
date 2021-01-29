mod audio;
mod audio_output;
mod audio_source;

use crate::audio_output::{play_queued_audio_system, AudioOutput};

use bevy::prelude::*;

pub use crate::audio_source::{AudioSource, Mp3Loader};
pub use audio::Audio;

#[derive(Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_thread_local_resource::<AudioOutput>()
            .add_asset::<AudioSource>()
            .init_asset_loader::<Mp3Loader>()
            .init_resource::<Audio>()
            .add_system_to_stage(stage::POST_UPDATE, play_queued_audio_system.system());
    }
}
