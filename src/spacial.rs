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

/// Configuration resource for spacial audio
///
/// If this resource is not added to the ECS, spacial audio is not applied.
#[derive(Resource)]
pub struct SpacialAudio {
    /// The volume will change from `1` at distance `0` to `0` at distance `max_distance`
    pub max_distance: f32,
}

impl SpacialAudio {
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

pub(crate) fn run_spacial_audio(
    spacial_audio: Res<SpacialAudio>,
    receiver: Query<&GlobalTransform, With<AudioReceiver>>,
    emitters: Query<(&GlobalTransform, &AudioEmitter)>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Ok(receiver_transform) = receiver.get_single() {
        spacial_audio.update(receiver_transform, &emitters, &mut audio_instances);
    }
}

pub(crate) fn cleanup_stopped_spacial_instances(
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
