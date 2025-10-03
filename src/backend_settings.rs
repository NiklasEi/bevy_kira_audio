use bevy::ecs::resource::Resource;
use bevy::utils::default;
use kira::{AudioManagerSettings, DefaultBackend, track::MainTrackBuilder};

/// This resource is used to configure the audio backend at creation
///
/// It needs to be inserted before adding the [`AudioPlugin`](crate::AudioPlugin) and will be
/// consumed by it. Settings cannot be changed at run-time!
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioSettings {
    /// The maximum number of sounds that can be playing at a time.
    pub sound_capacity: usize,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sound_capacity: 128,
        }
    }
}

impl From<AudioSettings> for AudioManagerSettings<DefaultBackend> {
    fn from(settings: AudioSettings) -> Self {
        AudioManagerSettings {
            main_track_builder: MainTrackBuilder::new().sound_capacity(settings.sound_capacity),
            ..default()
        }
    }
}
