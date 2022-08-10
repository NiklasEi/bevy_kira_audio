use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        // add our custom audio channel
        .add_audio_channel::<Background>()
        .add_startup_system(play)
        .run();
}

// Use the channel via the `AudioChannel<Background>` resource
fn play(background: Res<AudioChannel<Background>>, asset_server: Res<AssetServer>) {
    background.play(asset_server.load("sounds/loop.ogg"));
}

// Our type for the custom audio channel
struct Background;
