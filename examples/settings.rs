use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use kira::Panning;
use std::time::Duration;

/// This example shows the different settings that can be applied when playing a sound.
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, play_audio)
        .run();
}

/// Settings applied when playing a sound will overwrite the channel settings (like volume and panning)
fn play_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    // This is not nice to listen to, but demonstrates most settings
    audio
        .play(asset_server.load("sounds/loop.ogg"))
        // The first 0.5 seconds will not be looped and are the "intro"
        .loop_from(0.5)
        // The loop only goes until the 10th second
        .loop_until(10.0)
        // Fade-in with a dynamic easing
        .fade_in(AudioTween::new(
            Duration::from_secs(2),
            AudioEasing::OutPowi(2),
        ))
        // Only play on our right ear
        .with_panning(Panning(1.0))
        // Increase playback rate by 50% (this also increases the pitch)
        .with_playback_rate(1.5)
        // Play at half volume
        .with_volume(0.5)
        // play the track reversed
        .reverse();
}
