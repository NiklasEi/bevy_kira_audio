use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSettings, AudioSource};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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

fn prepare(asset_server: Res<AssetServer>, mut commands: Commands) {
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
