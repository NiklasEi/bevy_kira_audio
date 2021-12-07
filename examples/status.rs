use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin, InstanceHandle};

struct LoopAudioInstanceHandle {
    instance_handle: InstanceHandle,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(start_audio.system())
        .add_system(print_status.system())
        .add_system(process_keyboard_input.system())
        .run();
}

fn start_audio(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let asset_handle = asset_server.load("sounds/loop.ogg");
    let instance_handle = audio.play(asset_handle);
    println!("Audio started.");
    std::thread::sleep(std::time::Duration::from_secs(5));
    commands.insert_resource(LoopAudioInstanceHandle { instance_handle });
}

fn process_keyboard_input(
    audio: Res<Audio>,
    kb: Res<Input<KeyCode>>,
) {
    if kb.just_pressed(KeyCode::P) {
        audio.pause();
        println!("Audio paused.");
    } else if kb.just_pressed(KeyCode::S) {
        audio.stop();
        println!("Audio stopped.");
    } else if kb.just_pressed(KeyCode::R) {
        audio.resume();
        println!("Audio resumed.");
    }
}

fn print_status(audio: Res<Audio>, loop_audio: Res<LoopAudioInstanceHandle>) {
    let state = audio.state(loop_audio.instance_handle.clone());
    println!("Loop audio {:?}", state);
}
