#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_kira_audio::{AudioApp, AudioPlugin, Channel};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_channel::<Background>()
        .add_startup_system(play)
        .run();
}

fn play(channel: Res<Channel<Background>>, asset_server: Res<AssetServer>) {
    channel.play(asset_server.load("sounds/loop.ogg"));
}

struct Background;
