use bevy::ecs::resource::Resource;
use bevy::utils::default;
use kira::{AudioManagerSettings, Capacities, DefaultBackend, track::MainTrackBuilder};

/// This resource is used to configure the audio backend at creation
///
/// It needs to be inserted before adding the [`AudioPlugin`](crate::AudioPlugin) and will be
/// consumed by it. Settings cannot be changed at run-time!
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioSettings {
    /// The maximum number of sounds that can play on the main track; channel tracks set their own.
    pub sound_capacity: usize,
    /// The maximum number of sub-tracks on the main track; channel tracks set their own.
    pub sub_track_capacity: usize,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sound_capacity: 128,
            sub_track_capacity: 128,
        }
    }
}

impl From<AudioSettings> for AudioManagerSettings<DefaultBackend> {
    fn from(settings: AudioSettings) -> Self {
        AudioManagerSettings {
            capacities: Capacities {
                sub_track_capacity: settings.sub_track_capacity,
                ..default()
            },
            main_track_builder: MainTrackBuilder::new().sound_capacity(settings.sound_capacity),
            ..default()
        }
    }
}
