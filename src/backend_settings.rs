use bevy::ecs::resource::Resource;
use bevy::utils::default;
use kira::{AudioManagerSettings, Capacities, DefaultBackend};

/// This resource is used to configure the audio backend at creation
///
/// It needs to be inserted before adding the [`AudioPlugin`](crate::AudioPlugin) and will be
/// consumed by it. Settings cannot be changed at run-time!
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioSettings {
    /// Specifies how many of each resource type an audio context
    /// can have.
    pub capacities: Capacities,
}

impl From<AudioSettings> for AudioManagerSettings<DefaultBackend> {
    fn from(settings: AudioSettings) -> Self {
        AudioManagerSettings {
            capacities: settings.capacities,
            ..default()
        }
    }
}
