use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_kira_audio::prelude::*;

fn main() {
    App::new()
        .insert_resource(SpacialAudio { max_distance: 25. })
        .add_plugins((DefaultPlugins, AudioPlugin, CameraPlugin))
        .add_systems(Startup, setup)
        .run()
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Emitter Nr. 1
    let cooking = audio
        .play(asset_server.load("sounds/cooking.ogg"))
        .looped()
        .handle();
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/panStew.glb#Scene0"),
            transform: Transform::from_xyz(-5.0, 0., 0.),
            ..default()
        })
        .insert(AudioEmitter {
            instances: vec![cooking],
        });
    // Emitter Nr. 2
    let elevator_music = audio
        .play(asset_server.load("sounds/loop.ogg"))
        .looped()
        .handle();
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/boxOpen.glb#Scene0"),
            transform: Transform::from_xyz(10., 0., 0.),
            ..default()
        })
        .insert(AudioEmitter {
            instances: vec![elevator_music],
        });
    // Our camera will be the receiver
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.5, 10.0),
            ..default()
        })
        .insert(AudioReceiver)
        .insert(FlyCam);

    // Other scene setup...
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn( TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "WASD to move horizontally\nSPACE to ascend\nLSHIFT to descend\nESC to grab/release cursor.".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/monogram.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            }],
            ..default()
        },
        style: Style {
            margin: UiRect::all(Val::Px(15.)),
          ..default()
        },
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid {
            half_size: Vec3::new(25., 0., 25.),
        }),
        material: materials.add(Color::DARK_GREEN),
        ..default()
    });
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
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

fn cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(&mut window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
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
    if let Ok(window) = primary_window.get_single() {
        for mut transform in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                match window.cursor.grab_mode {
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

            transform.translation += velocity * time.delta_seconds() * SPEED;
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}

#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

/// Handles looking around if cursor is locked
fn player_look(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Ok(window) = primary_window.get_single() {
        let delta_state = state.as_mut();
        for mut transform in query.iter_mut() {
            for ev in delta_state.reader_motion.read(&motion) {
                match window.cursor.grab_mode {
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
