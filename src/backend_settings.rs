use bevy::ecs::resource::Resource;
use bevy::utils::default;
use kira::{AudioManagerSettings, Capacities, DefaultBackend};

/// This resource is used to configure the audio backend at creation
///
/// It needs to be inserted before adding the [`AudioPlugin`](crate::AudioPlugin) and will be
/// consumed by it. Settings cannot be changed at run-time!
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioSettings {
    /// The number of commands that can be sent to the audio backend at a time.
    ///
    /// Each action you take, like playing or pausing a sound
    /// queues up one command.
    ///
    /// Note that configuring a channel will cause one command per sound in the channel!
    pub command_capacity: usize,
    /// The maximum number of sounds that can be playing at a time.
    pub sound_capacity: u16,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            command_capacity: 128,
            sound_capacity: 128,
        }
    }
}

impl From<AudioSettings> for AudioManagerSettings<DefaultBackend> {
    fn from(settings: AudioSettings) -> Self {
        AudioManagerSettings {
            capacities: Capacities {
                send_track_capacity: settings.command_capacity,
                sub_track_capacity: settings.sound_capacity as usize,
                ..default()
            },
            ..default()
        }
    }
}
