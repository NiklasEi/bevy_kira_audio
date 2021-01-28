mod audio;
mod audio_output;

use crate::audio_output::{play_queued_audio_system, AudioOutput};

use bevy::prelude::*;

pub use audio::Audio;

#[derive(Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_thread_local_resource::<AudioOutput>()
            .init_resource::<Audio>()
            .add_system_to_stage(stage::POST_UPDATE, play_queued_audio_system.system());
    }
}
