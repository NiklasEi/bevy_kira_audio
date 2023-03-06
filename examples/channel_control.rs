use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

// This example demonstrates how to control an audio channel
// This kind of control is deferred to the end of the current frame update
// Left-click to pause the audio
// Right-click to resume the audio
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_system(play_loop.on_startup())
        .add_system(channel_control)
        .run()
}

fn channel_control(input: Res<Input<MouseButton>>, audio: Res<Audio>) {
    if input.just_pressed(MouseButton::Left) {
        audio.pause().fade_out(AudioTween::default());
    } else if input.just_pressed(MouseButton::Right) {
        audio.resume().fade_in(AudioTween::default());
    }
}

fn play_loop(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("sounds/loop.ogg")).looped();
    audio.play(asset_server.load("sounds/sound.ogg"));
}
