use bevy::prelude::*;
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_kira_audio::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(SpacialAudio { max_distance: 25. })
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(setup)
        .run()
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let cooking = audio
        .play(asset_server.load("sounds/cooking.ogg"))
        .with_volume(0.2)
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

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..default()
        })
        .insert(AudioReceiver)
        .insert(FlyCam);
}
