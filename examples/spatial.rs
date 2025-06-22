use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_kira_audio::{prelude::*, EmitterSettings};
use kira::Easing;

/// This example demonstrates the basic spatial audio support in `bevy_kira_audio`.
/// It adds `SpatialAudioPlugin` then spawns entities with `SpatialAudioEmitter`
/// and a receiver with the `SpatialAudioReceiver` component.
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AudioPlugin,
            SpatialAudioPlugin,
            CameraPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Emitter Nr. 1
    let pan_entity = commands
        .spawn((
            SceneRoot(asset_server.load("models/panStew.glb#Scene0")),
            Transform::from_xyz(-5.0, 0., 0.),
            SpatialAudioEmitter::default(),
        ))
        .id();
    // Emitter Nr. 2
    let music_box_entity = commands
        .spawn((
            SceneRoot(asset_server.load("models/boxOpen.glb#Scene0")),
            Transform::from_xyz(10., 0., 0.),
            SpatialAudioEmitter::default(),
            // Note that this is per-entity. To have emitters with different falloff values,
            // you would need to create child entities with their own emitters.
            EmitterSettings {
                distances: (2.0..=20.0).into(),
                // We will fully hear this entity within a distance of 2 units, and not hear it at
                // all by 20 units.
                attenuation_function: Easing::OutPowi(2), // A curve that starts fast and slows down
            },
        ))
        .id();
    let cooking_sound = asset_server.load("sounds/cooking.ogg");
    audio
        .play(cooking_sound)
        .with_emitter(pan_entity)
        .with_volume(0.0)
        .looped();

    // Play a non-spatial sound on the "Music" channel
    let music_sound = asset_server.load("sounds/loop.ogg");
    audio
        .play(music_sound)
        .with_emitter(music_box_entity)
        .looped()
        // Example of setting initial volume
        .with_volume(5.0);

    // Our camera will be the receiver.
    commands
        .spawn((Camera3d::default(), Transform::from_xyz(0.0, 0.5, 10.0)))
        .insert(SpatialAudioReceiver)
        .insert(FlyCam);

    // Other scene setup...
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn( (Text::new("WASD to move horizontally\nSPACE to ascend\nLSHIFT to descend\nESC to grab/release cursor."), TextFont{
        font:asset_server.load("fonts/monogram.ttf"),
        font_size: 40.0,
        ..default()
    }, TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    Node {margin: UiRect::all(Val::Px(15.)), ..default()}));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid {
            half_size: Vec3::new(25., 0., 25.),
        })),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(LinearRgba::GREEN))),
    ));
}
// only camera handling code from here on...

// Code modified from https://github.com/sburris0/bevy_flycam under the ISC License

// Copyright 2020 Spencer Burris
//
// Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted, provided that the above copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
struct CameraPlugin;

const SENSITIVITY: f32 = 0.00012;
const SPEED: f32 = 12.;

#[derive(Component)]
pub struct FlyCam;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_systems(Startup, initial_grab_cursor)
            .add_systems(Update, (player_move, player_look, cursor_grab));
    }
}

fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

fn cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(&mut window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor_options.grab_mode {
        CursorGrabMode::None => {
            window.cursor_options.grab_mode = CursorGrabMode::Confined;
            window.cursor_options.visible = false;
        }
        _ => {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Ok(window) = primary_window.single() {
        for mut transform in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                match window.cursor_options.grab_mode {
                    CursorGrabMode::None => (),
                    _ => match key {
                        KeyCode::KeyW => velocity += forward,
                        KeyCode::KeyS => velocity -= forward,
                        KeyCode::KeyA => velocity -= right,
                        KeyCode::KeyD => velocity += right,
                        KeyCode::Space => velocity += Vec3::Y,
                        KeyCode::ShiftLeft => velocity -= Vec3::Y,
                        _ => (),
                    },
                }
            }

            velocity = velocity.normalize_or_zero();

            transform.translation += velocity * time.delta_secs() * SPEED;
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}

#[derive(Resource, Default)]
struct InputState {
    pitch: f32,
    yaw: f32,
}

/// Handles looking around if cursor is locked
fn player_look(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    mut motion: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Ok(window) = primary_window.single() {
        let delta_state = state.as_mut();
        for mut transform in query.iter_mut() {
            for ev in motion.read() {
                match window.cursor_options.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        delta_state.pitch -= (SENSITIVITY * ev.delta.y * window_scale).to_radians();
                        delta_state.yaw -= (SENSITIVITY * ev.delta.x * window_scale).to_radians();
                    }
                }

                delta_state.pitch = delta_state.pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation = Quat::from_axis_angle(Vec3::Y, delta_state.yaw)
                    * Quat::from_axis_angle(Vec3::X, delta_state.pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}
