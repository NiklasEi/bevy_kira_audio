use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// This example demonstrates using dynamic audio channels. If you need a number of audio channels
/// that is not known at compile time, you can create and use dynamic channels based on string keys.
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, start_background_audio)
        .add_systems(Update, plop)
        .run()
}

fn start_background_audio(
    asset_server: Res<AssetServer>,
    mut audio: ResMut<DynamicAudioChannels>,
    mut commands: Commands,
) {
    audio
        .create_channel("example")
        .play(asset_server.load("sounds/loop.ogg"))
        .looped();
    commands.insert_resource(AudioHandle(asset_server.load("sounds/plop.ogg")));
}

fn plop(
    handle: Res<AudioHandle>,
    audio: Res<DynamicAudioChannels>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if input.just_pressed(MouseButton::Left) {
        audio.channel("example").play(handle.0.clone());
    }
}

#[derive(Resource)]
struct AudioHandle(Handle<AudioSource>);
