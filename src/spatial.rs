use crate::{AudioInstance, AudioTween};
use bevy::asset::{Assets, Handle};
use bevy::ecs::component::Component;
use bevy::prelude::{GlobalTransform, Query, Res, ResMut, Resource, With};

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

/// Configuration resource for global spatial audio radius
///
/// If neither this resource or the [`SpatialRadius`] component for entity exists in the ECS,
/// spatial audio is not applied.
#[derive(Resource)]
pub struct GlobalSpatialRadius {
    /// The volume will change from `1` at distance `0` to `0` at distance `radius`
    pub radius: f32,
}

/// Component for per-entity spatial audio radius
///
/// If neither this component or the [`GlobalSpatialRadius`] resource exists in the ECS, spatial
/// audio is not applied.
#[derive(Component)]
pub struct SpatialRadius {
    /// The volume will change from `1` at distance `0` to `0` at distance `radius`
    pub radius: f32,
}

pub(crate) fn run_spatial_audio(
    spatial_audio: Res<GlobalSpatialRadius>,
    receiver: Query<&GlobalTransform, With<AudioReceiver>>,
    emitters: Query<(&GlobalTransform, &AudioEmitter, Option<&SpatialRadius>)>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Ok(receiver_transform) = receiver.get_single() {
        for (emitter_transform, emitter, range) in emitters.iter() {
            let sound_path = emitter_transform.translation() - receiver_transform.translation();
            let volume = (1.
                - sound_path.length() / range.map_or(spatial_audio.radius, |r| r.radius))
            .clamp(0., 1.)
            .powi(2);

            let right_ear_angle = receiver_transform.right().angle_between(sound_path);
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

pub(crate) fn cleanup_stopped_spatial_instances(
    mut emitters: Query<&mut AudioEmitter>,
    instances: ResMut<Assets<AudioInstance>>,
) {
    emitters.iter_mut().for_each(|mut emitter| {
        emitter.instances.retain(|handle| {
            instances.get(handle).map_or(true, |instance| {
                !matches!(instance.handle.state(), kira::sound::PlaybackState::Stopped)
            })
        });
    });
}

pub(crate) fn spatial_audio_enabled(
    global: Option<Res<GlobalSpatialRadius>>,
    local: Query<(), With<SpatialRadius>>,
) -> bool {
    global.is_some() || !local.is_empty()
}
