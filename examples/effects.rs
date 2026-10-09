use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, (play, setup_text))
        .add_systems(Update, cycle_effects)
        .run();
}

fn play(audio: Res<Audio>, asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut cmd = audio.play(asset_server.load("sounds/loop.ogg"));

    // Effects run in the order they are added. The filter starts wide open, the reverb dry.
    let filter = cmd.add_effect(
        FilterBuilder::new()
            .mode(FilterMode::LowPass)
            .cutoff(20000.0)
            .mix(1.0_f32),
    );

    let reverb = cmd.add_effect(ReverbBuilder::new().feedback(0.8).damping(0.3).mix(0.0_f32));

    cmd.looped();

    commands.insert_resource(EffectsState {
        filter,
        reverb,
        step: 0,
        timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
    });
}

fn setup_text(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(Text::new(
        "Every three seconds the sound switches between dry, low-pass filter, reverb, and both",
    ));
}

fn cycle_effects(time: Res<Time>, mut state: ResMut<EffectsState>) {
    state.timer.tick(time.delta());
    if !state.timer.just_finished() {
        return;
    }

    let transition = AudioTween::linear(Duration::from_millis(200));

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
