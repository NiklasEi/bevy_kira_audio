use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

// This example demonstrates how to control a single audio instance
// This kind of control is not deferred to the end of the current frame. The audio command is
// sent to the audio thread immediately.
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, play_loop)
        .add_systems(Update, instance_control)
        .run()
}

fn instance_control(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        if input.just_pressed(MouseButton::Left) {
            match instance.state() {
                PlaybackState::Paused { .. } => {
                    // There are a lot of control methods defined on the instance
                    instance.resume(AudioTween::default());
                }
                PlaybackState::Playing { .. } => {
                    instance.pause(AudioTween::default());
                }
                _ => {}
            }
        }
    }
}

#[derive(Resource)]
struct InstanceHandle(Handle<AudioInstance>);

fn play_loop(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let handle = audio
        .play(asset_server.load("sounds/loop.ogg"))
        .looped()
        .handle();
    commands.insert_resource(InstanceHandle(handle));
}
