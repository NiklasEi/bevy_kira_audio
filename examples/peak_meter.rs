use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// A custom effect that measures how loud the audio is, and reports it back to the ECS.
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, (setup_ui, play))
        .add_systems(Update, update_meter)
        .run();
}

// -- The effect ------------------------------------------------------------------------------

/// The loudest sample seen since the value was last read, as `f32` bits.
///
/// `process` runs on the audio thread, so the level cannot be shared through a lock. An atomic
/// keeps both sides wait-free.
type SharedPeak = Arc<AtomicU32>;

struct PeakMeter {
    peak: SharedPeak,
}

impl Effect for PeakMeter {
    fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info) {
        let peak = input
            .iter()
            .map(|frame| frame.left.abs().max(frame.right.abs()))
            .fold(0.0f32, f32::max);

        // Bevy renders far slower than audio is processed, so several buffers pass between reads.
        // Accumulating the maximum makes sure short transients are not missed. For non-negative
        // floats the bit patterns order the same way the values do, so `fetch_max` works on them.
        self.peak.fetch_max(peak.to_bits(), Ordering::Relaxed);
    }
}

/// Configures a [`PeakMeter`].
struct PeakMeterBuilder;

impl AudioEffect for PeakMeterBuilder {
    type Handle = PeakMeterHandle;

    fn build_effect(self) -> (Box<dyn Effect>, Self::Handle) {
        let peak = SharedPeak::default();

        (
            Box::new(PeakMeter { peak: peak.clone() }),
            PeakMeterHandle(peak),
        )
    }
}

/// Reads the measured level.
#[derive(Resource)]
struct PeakMeterHandle(SharedPeak);

impl PeakMeterHandle {
    /// The loudest sample since this method was last called, where `1.0` is full scale.
    fn take_peak(&self) -> f32 {
        f32::from_bits(self.0.swap(0, Ordering::Relaxed))
    }
}

// -- Playback --------------------------------------------------------------------------------

fn play(audio: Res<Audio>, asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut sound = audio.play(asset_server.load("sounds/loop.ogg"));
    let meter = sound.add_effect(PeakMeterBuilder);
    sound.looped();

    commands.insert_resource(meter);
}

// -- The meter -------------------------------------------------------------------------------

/// The quietest level the bar shows. Anything below this reads as empty.
const FLOOR_DB: f32 = -60.0;

/// How quickly the bar falls back down, the way the needle of a real meter drops.
const FALL_DB_PER_SECOND: f32 = 36.0;

#[derive(Component)]
struct MeterBar {
    /// Displayed level in dBFS, where `0.0` is full scale.
    level_db: f32,
}

impl Default for MeterBar {
    fn default() -> Self {
        Self { level_db: FLOOR_DB }
    }
}

#[derive(Component)]
struct MeterLabel;

fn update_meter(
    time: Res<Time>,
    meter: Res<PeakMeterHandle>,
    mut bar: Single<(&mut MeterBar, &mut Node, &mut BackgroundColor)>,
    mut label: Single<&mut Text, With<MeterLabel>>,
) {
    let (state, node, color) = &mut *bar;

    // Audio is rendered in bursts, so some frames see no new samples at all. Levels are shown on
    // a decibel scale, because that is how loudness is perceived: a linear bar spends almost all
    // of its length on the loudest few decibels and looks broken for normal material.
    let peak = meter.take_peak();
    let peak_db = if peak > 0.0 {
        20.0 * peak.log10()
    } else {
        FLOOR_DB
    };

    // Jump straight to a new peak, but ease back down, the way a real meter behaves.
    state.level_db = if peak_db > state.level_db {
        peak_db
    } else {
        (state.level_db - FALL_DB_PER_SECOND * time.delta_secs()).max(FLOOR_DB)
    };

    let fill = ((state.level_db - FLOOR_DB) / -FLOOR_DB).clamp(0.0, 1.0);
    node.width = Val::Percent(fill * 100.0);
    color.0 = if state.level_db > -3.0 {
        Color::linear_rgb(0.9, 0.2, 0.2)
    } else if state.level_db > -12.0 {
        Color::linear_rgb(0.9, 0.8, 0.3)
    } else {
        Color::linear_rgb(0.3, 0.85, 0.4)
    };

    **label = Text::new(if state.level_db > FLOOR_DB {
        format!("{:.1} dBFS", state.level_db)
    } else {
        "-inf dBFS".to_owned()
    });
}

// -- UI --------------------------------------------------------------------------------------

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let font = TextFont {
        font: asset_server.load("fonts/monogram.ttf").into(),
        font_size: 32.0.into(),
        ..default()
    };

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(60.0)),
            row_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Peak meter (custom effect)"),
                font.clone(),
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(format!("{FLOOR_DB:.0} dBFS to 0 dBFS")),
                font.clone(),
                TextColor(Color::linear_rgb(0.45, 0.45, 0.45)),
            ));

            // The track the bar grows inside of.
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(48.0),
                        ..default()
                    },
                    BackgroundColor(Color::linear_rgb(0.12, 0.12, 0.14)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::linear_rgb(0.3, 0.85, 0.4)),
                        MeterBar::default(),
                    ));
                });

            parent.spawn((
                Text::new("-inf dBFS"),
                font,
                TextColor(Color::linear_rgb(0.6, 0.6, 0.6)),
                MeterLabel,
            ));
        });
}
