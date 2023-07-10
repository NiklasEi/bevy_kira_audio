use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// This example shows how to load a sound file with applied audio settings.
/// You can also easily apply settings when playing a sound (see the `settings` example).
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, play_audio)
        .run();
}

fn play_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("sounds/loop_with_settings.ogg.ron"));
}
