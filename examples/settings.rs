use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

/// This example shows the different settings that can be applied when first playing a sound.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(play_audio)
        .run();
}

fn play_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(asset_server.load("sounds/loop.ogg"))
        .loop_from(0.5)
        .fade_in(AudioTween::new(
            Duration::from_secs(2),
            AudioEasing::OutPowi(2),
        ))
        .with_panning(1.0)
        .with_playback_rate(1.5)
        .with_volume(0.5)
        .reverse();
}
