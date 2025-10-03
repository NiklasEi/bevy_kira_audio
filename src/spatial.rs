use crate::AudioOutput;
use bevy::app::{App, Plugin, PostUpdate};
use bevy::ecs::component::{Component, HookContext};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::Query;
use bevy::ecs::world::DeferredWorld;
use bevy::log::warn;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::transform::TransformSystem;
use kira::track::{SpatialTrackBuilder, SpatialTrackDistances};
use kira::{Easing, Tween};

/// This plugin adds basic spatial audio.
///
/// Add `SpatialAudioEmitter` components to entities that emit spacial audio.
/// One entity, usually the "Player" or the Camera should get the `SpatialAudioReceiver` component.
///
/// See the `spacial` example of `bevy_kira_audio`.
pub struct SpatialAudioPlugin;

impl Plugin for SpatialAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (update_listener_transform, update_emitter_positions)
                // The entire chain runs after Bevy's transform propagation.
                .after(TransformSystem::TransformPropagate),
        );
    }
}

/// Component for audio emitters
///
/// Add `EmitterSettings` to control distance and attenuation
/// and receiver positions.
#[derive(Component, Default)]
#[require(Transform)]
#[component(on_add=emitter_added)]
pub struct SpatialAudioEmitter {
    /// The handle to the Kira spatial track associated with this emitter.
    /// This is created automatically when the component is added.
    pub track: Option<kira::track::SpatialTrackHandle>,
}

/// Component for the spatial audio receiver.
///
/// Most likely you will want to add this component to your player or you camera.
/// There can only ever be one entity with this component at a given time!
#[derive(Component)]
#[require(Transform)]
pub struct SpatialAudioReceiver;

/// A component to define spatial properties for an audio emitter
/// that must be set at creation time.
#[derive(Component, Clone)]
pub struct EmitterSettings {
    /// The distances from a listener at which the emitter is loudest and quietest.
    /// Full volume at `min_distance`, silent at `max_distance`.
    pub distances: SpatialTrackDistances,
    /// The curve used for volume attenuation over the specified distances.
    pub attenuation_function: Easing,
}

impl Default for EmitterSettings {
    fn default() -> Self {
        Self {
            distances: SpatialTrackDistances::default(),
            attenuation_function: Easing::Linear,
        }
    }
}
/// This hook runs whenever a `SpatialAudioEmitter` component is added to an entity.
fn emitter_added(mut world: DeferredWorld, context: HookContext) {
    // We need to get the GlobalTransform to set the initial position.
    let transform = world
        .get::<GlobalTransform>(context.entity)
        .cloned()
        .unwrap_or_default();
    // Check if the entity also has the EmitterSettings component.
    let emitter_settings = world
        .get::<EmitterSettings>(context.entity)
        .cloned()
        .unwrap_or_default(); // Fall back to default settings if it doesn't.

    let bevy_pos = transform.translation();
    let mint_pos = mint::Vector3 {
        x: bevy_pos.x,
        y: bevy_pos.y,
        z: bevy_pos.z,
    };

    let listener_id = {
        let Some(audio_output) = world.get_non_send_resource::<AudioOutput>() else {
            return;
        };
        let Some(listener_handle) = audio_output.listener.as_ref() else {
            warn!("Cannot initialize spatial emitter: No listener found.");
            return;
        };
        listener_handle.id() // Copy the ID
    };

    if let Some(mut audio_output) = world.get_non_send_resource_mut::<AudioOutput>() {
        let Some(manager) = audio_output.manager.as_mut() else {
            return;
        };

        // Create a builder and apply the settings from the component.
        let builder = SpatialTrackBuilder::new()
            .distances(emitter_settings.distances) // Set the falloff distance
            .attenuation_function(emitter_settings.attenuation_function); // Set the falloff curve

        match manager.add_spatial_sub_track(listener_id, mint_pos, builder) {
            Ok(track_handle) => {
                if let Some(mut emitter) = world.get_mut::<SpatialAudioEmitter>(context.entity) {
                    emitter.track = Some(track_handle);
                }
            }
            Err(e) => {
                warn!(
                    "Error creating spatial track for entity {:?}: {:?}",
                    context.entity, e
                );
            }
        }
    }
}
fn update_listener_transform(
    mut audio_output: bevy::ecs::system::NonSendMut<AudioOutput>,
    receiver_query: Query<&GlobalTransform, With<SpatialAudioReceiver>>,
) {
    let Some(listener) = audio_output.listener.as_mut() else {
        return;
    };

    if let Ok(receiver_transform) = receiver_query.single() {
        let pos = receiver_transform.translation();
        let rot = receiver_transform.rotation();
        let mint_pos = mint::Vector3 {
            x: pos.x,
            y: pos.y,
            z: pos.z,
        };
        let mint_rot = mint::Quaternion {
            v: mint::Vector3 {
                x: rot.x,
                y: rot.y,
                z: rot.z,
            },
            s: rot.w,
        };

        listener.set_position(mint_pos, Tween::default());
        listener.set_orientation(mint_rot, Tween::default());
    }
}

fn update_emitter_positions(mut query: Query<(&mut SpatialAudioEmitter, &GlobalTransform)>) {
    for (mut emitter, transform) in &mut query {
        if let Some(track) = emitter.track.as_mut() {
            let pos = transform.translation();
            let mint_pos = mint::Vector3 {
                x: pos.x,
                y: pos.y,
                z: pos.z,
            };
            track.set_position(mint_pos, Tween::default());
        }
    }
}
