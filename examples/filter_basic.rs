use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use kira::effect::filter::{FilterBuilder, FilterMode};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, play)
        .run();
}

fn play(audio: Res<Audio>, asset_server: Res<AssetServer>) {
    audio
        .play(asset_server.load("sounds/loop.ogg"))
        .with_effect(FilterBuilder::new().mode(FilterMode::LowPass).cutoff(300.0))
        .looped();
}
