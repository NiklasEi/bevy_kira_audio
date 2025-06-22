use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use kira::Capacities;

/// This example needs to be played in release mode! `cargo run --example stress_test --release`
/// A large amount (100) of sounds will be played in every frame.
///
/// The main objective here is to demonstrate that the plugin and Kira can handle
/// large sound volumes over a longer period of time.
///
/// Depending on your machine, the number of sounds you can play before audio issues appear may differ.
fn main() {
    App::new()
        // We need to increase the queue sizes of the audio backend.
        // TODO::
        .insert_resource(AudioSettings {
            capacities: Capacities::default(),
        })
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, prepare)
        .add_systems(Update, (check, play))
        .run();
}

#[derive(Resource)]
struct LoadingAudioHandle(Handle<AudioSource>);

#[derive(Resource)]
struct AudioHandle(Handle<AudioSource>);

fn prepare(asset_server: Res<AssetServer>, mut commands: Commands, audio: Res<Audio>) {
    // Stop our ears from exploding...
    // Playing multiple sounds in the same frame can get quite loud
    audio.set_volume(0.001);
    commands.insert_resource(LoadingAudioHandle(asset_server.load("sounds/plop.ogg")));

    commands.spawn(Camera2d);
    commands.spawn(Text::new(
        r#"
    This is a stress test playing 100 sounds every frame

    Milage may vary; be sure to run in release mode!"#,
    ));
}

fn check(
    handle: Option<Res<LoadingAudioHandle>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if let Some(handle) = handle {
        if asset_server
            .get_load_state(handle.0.id())
            .map(|state| state.is_loaded())
            .unwrap_or(false)
        {
            commands.insert_resource(AudioHandle(handle.0.clone()));
            commands.remove_resource::<LoadingAudioHandle>();
        }
    }
}

fn play(handle: Option<Res<AudioHandle>>, audio: Res<Audio>) {
    if let Some(handle) = handle {
        // The max number here depends on your hardware.
        // If you get warnings and/or stuttered sounds try reducing the amount and/or changing the
        // capacities of the `AudioSettings` in the `main` method.
        for _ in 0..100 {
            audio.play(handle.0.clone());
        }
    }
}
