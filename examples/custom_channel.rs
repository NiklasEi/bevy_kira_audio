use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_kira_audio::{
    AudioBundle, AudioChannel, AudioChannelBundle, ChannelState, LoopSettings, PlaybackSettings,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        // add our custom audio channel
        .add_systems(Startup, play)
        .add_systems(Update, on_click_single)
        .run();
}

// Use the channel via the `AudioChannel<Background>` resource
fn play(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(AudioChannelBundle::<Background>::default());
    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/loop.ogg"),
            settings: PlaybackSettings {
                loop_settings: Some(LoopSettings::default()),
                ..default()
            },
            ..default()
        },
        AudioChannel::<Background>::default(),
    ));
}

fn on_click_single(mut audio: Query<&mut AudioInstance>, input: Res<Input<MouseButton>>) {
    let Ok(mut audio) = audio.get_single_mut() else {
        return;
    };
    if input.just_pressed(MouseButton::Left) {
        audio.pause(AudioTween::default());
    }
    if input.just_pressed(MouseButton::Right) {
        audio.resume(AudioTween::default());
    }
}

fn on_click_channel(
    mut channel: Query<&mut ChannelState, With<AudioChannel<Background>>>,
    input: Res<Input<MouseButton>>,
) {
    let Ok(mut audio_channel) = channel.get_single_mut() else {
        return;
    };
    if input.just_pressed(MouseButton::Left) {
        audio_channel.paused = true;
    }
    if input.just_pressed(MouseButton::Right) {
        audio_channel.paused = false;
    }
}

// Our type for the custom audio channel
#[derive(Component)]
struct Background;
