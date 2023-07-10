use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, play_loop)
        .run()
}

// `Audio` is an alias for `AudioChannel<MainTrack>`, which is the default channel added by the audio plugin
// See the `custom_channel` example to add your own audio channels
fn play_loop(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("sounds/loop.ogg")).looped();
}
