use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use kira::Easing;
use std::time::Duration;

// This example demonstrates how to control an audio channel
// This kind of control is deferred to the end of the current frame update
// Left-click to pause the audio
// Right-click to resume the audio
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, play_loop)
        .add_systems(Update, channel_control)
        .run();
}

fn channel_control(input: Res<ButtonInput<MouseButton>>, audio: Res<Audio>) {
    if input.just_pressed(MouseButton::Left) {
        audio
            .pause()
            .fade_out(AudioTween::new(Duration::from_secs(2), Easing::Linear));
    } else if input.just_pressed(MouseButton::Right) {
        audio
            .resume()
            .fade_in(AudioTween::new(Duration::from_secs(2), Easing::Linear));
    }
}

fn play_loop(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("sounds/loop.ogg")).looped();
    audio.play(asset_server.load("sounds/sound.ogg"));
    commands.spawn(Camera2d);
    commands.spawn(Text::new(
        r#"
    This example demonstrates how to control an audio channel
    The audio commands have 2s linear fade in/out

    Left-click to pause the audio
    Right-click to resume the audio"#,
    ));
}
