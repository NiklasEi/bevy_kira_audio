use crate::{AudioInstance, AudioTween};
use bevy::asset::{Assets, Handle};
use bevy::ecs::component::Component;
use bevy::math::Vec3;
use bevy::prelude::{GlobalTransform, Query, Res, ResMut, Resource, With};
use std::f32::consts::PI;

/// Component for audio emitters
///
/// Add [`Handle<AudioInstance>`]s to control their pan and volume based on emitter
/// and receiver positions.
#[derive(Component, Default)]
pub struct AudioEmitter {
    /// Audio instances that are played by this emitter
    ///
    /// The same instance should only be on one emitter.
    pub instances: Vec<Handle<AudioInstance>>,
}

/// Component for the audio receiver
///
/// Most likely you will want to add this component to your player or you camera.
/// The entity needs a [`Transform`] and [`GlobalTransform`]. The view direction of the [`GlobalTransform`]
/// will
#[derive(Component)]
pub struct AudioReceiver;

/// Configuration resource for spatial audio
///
/// If this resource is not added to the ECS, spatial audio is not applied.
#[derive(Resource)]
pub struct SpatialAudio {
    /// The volume will change from `1` at distance `0` to `0` at distance `max_distance`
    pub max_distance: f32,
}

impl SpatialAudio {
    pub(crate) fn update(
        &self,
        receiver_transform: &GlobalTransform,
        emitters: &Query<(&GlobalTransform, &AudioEmitter)>,
        audio_instances: &mut Assets<AudioInstance>,
    ) {
        for (emitter_transform, emitter) in emitters {
            let sound_path = emitter_transform.translation() - receiver_transform.translation();
            let volume = (1. - sound_path.length() / self.max_distance)
                .clamp(0., 1.)
                .powi(2);

            let right_ear_angle = if sound_path == Vec3::ZERO {
                PI / 2.
            } else {
                receiver_transform.right().angle_between(sound_path)
            };
            let panning = (right_ear_angle.cos() + 1.) / 2.;

            for instance in emitter.instances.iter() {
                if let Some(instance) = audio_instances.get_mut(instance) {
                    instance.set_volume(volume as f64, AudioTween::default());
                    instance.set_panning(panning as f64, AudioTween::default());
                }
            }
        }
    }
}

pub(crate) fn run_spatial_audio(
    spatial_audio: Res<SpatialAudio>,
    receiver: Query<&GlobalTransform, With<AudioReceiver>>,
    emitters: Query<(&GlobalTransform, &AudioEmitter)>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Ok(receiver_transform) = receiver.get_single() {
        spatial_audio.update(receiver_transform, &emitters, &mut audio_instances);
    }
}

pub(crate) fn cleanup_stopped_spatial_instances(
    mut emitters: Query<&mut AudioEmitter>,
    instances: ResMut<Assets<AudioInstance>>,
) {
    for mut emitter in emitters.iter_mut() {
        let handles = &mut emitter.instances;

        handles.retain(|handle| {
            if let Some(instance) = instances.get(handle) {
                instance.handle.state() != kira::sound::PlaybackState::Stopped
            } else {
                true
            }
        });
    }
}
