use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// This example needs to be played in release mode!
/// A large amount of sounds will be played in every frame.
///
/// The main objective here is to demonstrate that the plugin and Kira can handle
/// large sound volumes over a longer period of time.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // We need to increase the queue sizes of the audio backend.
        // The default is 128 per queue, which is way too low for playing as many sounds
        // as this example does.
        .insert_resource(AudioSettings {
            sound_capacity: 4096,
            command_capacity: 1024,
        })
        .add_plugin(AudioPlugin)
        .add_startup_system(prepare)
        .add_system(check)
        .add_system(play)
        .run()
}

struct MyLoadingHandle(Handle<AudioSource>);
struct MyHandle(Handle<AudioSource>);

fn prepare(asset_server: Res<AssetServer>, mut commands: Commands, audio: Res<Audio>) {
    // Stop our ears from exploding...
    // Playing multiple sounds in the same frame just sums up their volume
    audio.set_volume(0.001);
    commands.insert_resource(MyLoadingHandle(asset_server.load("sounds/plop.ogg")))
}

fn check(
    handle: Option<Res<MyLoadingHandle>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if let Some(handle) = handle {
        if asset_server.get_load_state(handle.0.id) == LoadState::Loaded {
            commands.insert_resource(MyHandle(handle.0.clone()));
            commands.remove_resource::<MyLoadingHandle>();
        }
    }
}

fn play(handle: Option<Res<MyHandle>>, audio: Res<Audio>) {
    if let Some(handle) = handle {
        for _ in 0..75 {
            audio.play(handle.0.clone());
        }
    }
}
