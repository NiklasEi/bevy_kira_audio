use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use kira::effect::filter::{FilterBuilder, FilterHandle, FilterMode};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, play)
        .add_systems(Update, toggle_filter)
        .run();
}

fn play(audio: Res<Audio>, asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut cmd = audio.play(asset_server.load("sounds/loop.ogg"));
    let filter = cmd.add_effect(
        FilterBuilder::new()
            .mode(FilterMode::LowPass)
            .cutoff(20000.0) // start fully open
            .mix(1.0_f32),
    );
    cmd.looped();

    commands.insert_resource(FilterState {
        handle: filter,
        muffled: false,
        timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
    });
}

fn toggle_filter(time: Res<Time>, mut state: ResMut<FilterState>) {
    state.timer.tick(time.delta());
    if !state.timer.just_finished() {
        return;
    }

    state.muffled = !state.muffled;
    let transition = kira::Tween {
        duration: Duration::from_millis(150),
        ..default()
    };

    if state.muffled {
        // Strong low-pass: only bass frequencies come through
        state.handle.set_cutoff(300.0, transition);
        info!("Filter ON (muffled)");
    } else {
        // Wide open: full spectrum
        state.handle.set_cutoff(20000.0, transition);
        info!("Filter OFF (clear)");
    }
}

#[derive(Resource)]
struct FilterState {
    handle: FilterHandle,
    muffled: bool,
    timer: Timer,
}
