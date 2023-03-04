use bevy::prelude::*;
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_kira_audio::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(SpacialAudio { max_distance: 25. })
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(NoCameraPlayerPlugin)
        .add_system(setup.on_startup())
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
        mesh: meshes.add(
            shape::Plane {
                size: 50.,
                subdivisions: 0,
            }
            .into(),
        ),
        material: materials.add(Color::DARK_GREEN.into()),
        ..default()
    });
}
