use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use kira::effect::delay::DelayBuilder;
use kira::effect::filter::{FilterBuilder, FilterHandle, FilterMode};
use kira::effect::reverb::{ReverbBuilder, ReverbHandle};
use std::time::Duration;

/// Demonstrates applying three effects to a single audio instance,
/// then toggling them on and off at runtime.
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, play)
        .add_systems(Update, cycle_effects)
        .run();
}

fn play(audio: Res<Audio>, asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut cmd = audio.play(asset_server.load("sounds/loop.ogg"));

    // Effect 1: low-pass filter (starts wide open)
    let filter = cmd.add_effect(
        FilterBuilder::new()
            .mode(FilterMode::LowPass)
            .cutoff(20000.0)
            .mix(1.0_f32),
    );

    // Effect 2: delay (starts silent)
    cmd.with_effect(
        DelayBuilder::new()
            .delay_time(Duration::from_millis(350))
            .feedback(0.4_f32)
            .mix(0.0_f32),
    );

    // Effect 3: reverb (starts dry)
    let reverb = cmd.add_effect(ReverbBuilder::new().feedback(0.8).damping(0.3).mix(0.0_f32));

    cmd.looped();

    commands.insert_resource(EffectsState {
        filter,
        reverb,
        step: 0,
        timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
    });
}

fn cycle_effects(time: Res<Time>, mut state: ResMut<EffectsState>) {
    state.timer.tick(time.delta());
    if !state.timer.just_finished() {
        return;
    }

    let transition = kira::Tween {
        duration: Duration::from_millis(200),
        ..default()
    };

    // Cycle through: dry -> filter -> reverb -> filter+reverb -> dry
    state.step = (state.step + 1) % 4;
    match state.step {
        0 => {
            state.filter.set_cutoff(20000.0, transition);
            state.reverb.set_mix(0.0_f32, transition);
            info!("All effects OFF (dry)");
        }
        1 => {
            state.filter.set_cutoff(400.0, transition);
            state.reverb.set_mix(0.0_f32, transition);
            info!("Filter ON (muffled)");
        }
        2 => {
            state.filter.set_cutoff(20000.0, transition);
            state.reverb.set_mix(0.6_f32, transition);
            info!("Reverb ON");
        }
        3 => {
            state.filter.set_cutoff(400.0, transition);
            state.reverb.set_mix(0.6_f32, transition);
            info!("Filter + Reverb ON");
        }
        _ => unreachable!(),
    }
}

#[derive(Resource)]
struct EffectsState {
    filter: FilterHandle,
    reverb: ReverbHandle,
    step: usize,
    timer: Timer,
}
