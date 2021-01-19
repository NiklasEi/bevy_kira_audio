mod audio;
mod audio_output;
mod audio_source;
mod channel;

pub use audio::*;
pub use audio_output::*;
pub use audio_source::*;
use bevy::prelude::{stage, AddAsset, AppBuilder, IntoSystem, Plugin};

pub mod prelude {
    pub use crate::{Audio, AudioOutput, AudioSource, Decodable};
}

/// Adds support for audio playback to an App
#[derive(Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_asset_loader::<AudioLoader>()
            .init_thread_local_resource::<AudioOutput<AudioSource>>()
            .init_resource::<Audio<AudioSource>>()
            .add_asset::<AudioSource>()
            .add_system_to_stage(
                stage::POST_UPDATE,
                play_queued_audio_system::<AudioSource>.system(),
            );
    }
}
