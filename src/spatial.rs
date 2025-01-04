use crate::{AudioInstance, AudioSystemSet, AudioTween};
use bevy::app::{App, Plugin, PostUpdate, PreUpdate};
use bevy::asset::{Assets, Handle};
use bevy::ecs::component::Component;
use bevy::ecs::{
    change_detection::{Res, ResMut},
    query::With,
    schedule::IntoSystemConfigs,
    system::{Query, Resource},
};
use bevy::math::{Curve, Vec3};
use bevy::prelude::{EaseFunction, EasingCurve};
use bevy::transform::components::{GlobalTransform, Transform};
use std::f32::consts::PI;

/// This plugin adds basic spatial audio.
///
/// Add `SpatialAudioEmitter` components to entities that emit spacial audio.
/// One entity, usually the "Player" or the Camera should get the `SpatialAudioReceiver` component.
///
/// See the `spacial` example of `bevy_kira_audio`.
pub struct SpatialAudioPlugin;

impl Plugin for SpatialAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DefaultSpatialRadius>()
            .add_systems(
                PreUpdate,
                cleanup_stopped_spatial_instances.in_set(AudioSystemSet::InstanceCleanup),
            )
            .add_systems(PostUpdate, run_spatial_audio);
    }
}

/// Component for audio emitters
///
/// Add [`Handle<AudioInstance>`]s to control their pan and volume based on emitter
/// and receiver positions.
#[derive(Component)]
#[require(Transform, SpatialDampingCurve)]
pub struct SpatialAudioEmitter {
    /// Audio instances that are played by this emitter
    ///
    /// The same instance should only be on one emitter.
    pub instances: Vec<Handle<AudioInstance>>,
}

/// Component for the spatial audio receiver.
///
/// Most likely you will want to add this component to your player or you camera.
/// There can only ever be one entity with this component at a given time!
#[derive(Component)]
#[require(Transform)]
pub struct SpatialAudioReceiver;

/// Configuration resource for global spatial audio radius
///
/// This resource has to exist for spatial audio and will be initialized by the `SpatialAudioPlugin`.
/// If an emitter does not have a `SpatialRadius`, the `GlobalSpatialRadius` is used.
#[derive(Resource)]
pub struct DefaultSpatialRadius {
    /// The volume will change from `1` at distance `0` to `0` at distance `radius`
    pub radius: f32,
}

impl Default for DefaultSpatialRadius {
    fn default() -> Self {
        Self { radius: 25.0 }
    }
}

/// Component for per-entity spatial audio radius
///
/// If an emitter does not have this component, the [`DefaultSpatialRadius`] is used instead.
#[derive(Component)]
pub struct SpatialRadius {
    /// The volume will change from `1` at distance `0` to `0` at distance `radius`
    pub radius: f32,
}

#[derive(Component)]
struct SpatialDampingCurve(EaseFunction);

impl Default for SpatialDampingCurve {
    fn default() -> Self {
        SpatialDampingCurve(EaseFunction::Linear)
    }
}

fn run_spatial_audio(
    spatial_audio: Res<DefaultSpatialRadius>,
    receiver: Query<&GlobalTransform, With<SpatialAudioReceiver>>,
    emitters: Query<(
        &GlobalTransform,
        &SpatialAudioEmitter,
        &SpatialDampingCurve,
        Option<&SpatialRadius>,
    )>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Ok(receiver_transform) = receiver.get_single() {
        for (emitter_transform, emitter, damping_curve, range) in emitters.iter() {
            let sound_path = emitter_transform.translation() - receiver_transform.translation();
            let progress = (sound_path.length() / range.map_or(spatial_audio.radius, |r| r.radius))
                .clamp(0., 1.);
            let volume: f32 = 1.
                - EasingCurve::new(0., 1., damping_curve.0)
                    .sample_unchecked(progress)
                    .clamp(0., 1.);

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

fn cleanup_stopped_spatial_instances(
    mut emitters: Query<&mut SpatialAudioEmitter>,
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
