use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(load_audio)
        .run();
}

fn load_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play_looped(asset_server.load("sounds/semantic_duration.ogg.ron"));
}
