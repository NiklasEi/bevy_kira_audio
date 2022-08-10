use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(play_audio)
        .run();
}

fn play_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("sounds/loop_with_settings.ogg.ron"));
}
