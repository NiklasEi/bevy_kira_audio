use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use kira::effect::reverb::{ReverbBuilder, ReverbHandle};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, play)
        .add_systems(Update, toggle_reverb)
        .run();
}

fn play(audio: Res<Audio>, asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut cmd = audio.play(asset_server.load("sounds/cooking.ogg"));
    let reverb = cmd.add_effect(
        ReverbBuilder::new()
            .feedback(0.85)
            .damping(0.2)
            .mix(0.0_f32), // start dry
    );
    cmd.looped();

    commands.insert_resource(ReverbState {
        handle: reverb,
        on: false,
        timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
    });
}

fn toggle_reverb(time: Res<Time>, mut state: ResMut<ReverbState>) {
    state.timer.tick(time.delta());
    if !state.timer.just_finished() {
        return;
    }

    state.on = !state.on;
    let transition = kira::Tween {
        duration: Duration::from_millis(80),
        ..default()
    };

    if state.on {
        state.handle.set_mix(0.7_f32, transition);
        info!("Reverb ON");
    } else {
        state.handle.set_mix(0.0_f32, transition);
        info!("Reverb OFF");
    }
}

#[derive(Resource)]
struct ReverbState {
    handle: ReverbHandle,
    on: bool,
    timer: Timer,
}
