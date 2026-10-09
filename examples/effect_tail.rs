use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

/// Reverb and delay keep sounding after the sound feeding them has stopped. A sound with
/// per-instance effects plays on a track of its own, and once that track goes away so does
/// everything still ringing on it, so `with_effect_tail` decides how long it sticks around.
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .init_resource::<Ringing>()
        .add_systems(Startup, setup)
        .add_systems(Update, (play, update_status).chain())
        .run();
}

const TAILS: [(KeyCode, Duration, &str); 3] = [
    (KeyCode::Digit1, Duration::ZERO, "[1] no tail"),
    (KeyCode::Digit2, Duration::from_millis(500), "[2] 500 ms"),
    (KeyCode::Digit3, Duration::from_secs(2), "[3] 2 s"),
];

const COLOR_ALIVE: Color = Color::linear_rgb(0.3, 0.85, 0.4);
const COLOR_GONE: Color = Color::linear_rgb(0.45, 0.45, 0.45);

#[derive(Resource)]
struct Plop(Handle<AudioSource>);

/// The plop that was played last, and how much of its tail is left.
#[derive(Resource, Default)]
struct Ringing {
    instance: Option<Handle<AudioInstance>>,
    label: &'static str,
    tail: Duration,
    /// Counts down in real time once the plop itself has stopped. `None` while it is still playing.
    left: Option<Duration>,
}

#[derive(Component)]
struct StatusText;

fn play(
    keys: Res<ButtonInput<KeyCode>>,
    audio: Res<Audio>,
    plop: Res<Plop>,
    mut ringing: ResMut<Ringing>,
) {
    for (key, tail, label) in TAILS {
        if !keys.just_pressed(key) {
            continue;
        }

        let mut cmd = audio.play(plop.0.clone());
        cmd.with_effect(
            ReverbBuilder::new()
                .feedback(0.85)
                .damping(0.2)
                .mix(0.85_f32),
        )
        .with_effect_tail(tail);

        *ringing = Ringing {
            instance: Some(cmd.handle()),
            label,
            tail,
            left: None,
        };
    }
}

fn update_status(
    time: Res<Time<Real>>,
    audio: Res<Audio>,
    mut ringing: ResMut<Ringing>,
    mut status: Single<(&mut Text, &mut TextColor), With<StatusText>>,
) {
    let Some(instance) = ringing.instance.clone() else {
        return;
    };

    // The tail is counted from the moment the sound stops, not from the key press.
    match ringing.left {
        Some(left) => ringing.left = Some(left.saturating_sub(time.delta())),
        None if audio.state(&instance) == PlaybackState::Stopped => {
            ringing.left = Some(ringing.tail)
        }
        None => {}
    }

    let (state, alive) = match ringing.left {
        None => ("plop playing".to_owned(), true),
        Some(left) if left.is_zero() => ("track gone".to_owned(), false),
        Some(left) => (format!("track alive {:.2}s more", left.as_secs_f32()), true),
    };

    let (text, color) = &mut *status;
    **text = Text::new(format!("{}  -  {}", ringing.label, state));
    color.0 = if alive { COLOR_ALIVE } else { COLOR_GONE };
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.insert_resource(Plop(asset_server.load("sounds/plop.ogg")));

    let font = asset_server.load("fonts/monogram.ttf");
    let normal = TextFont {
        font: font.clone().into(),
        font_size: 26.0.into(),
        ..default()
    };

    commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(40.0)),
            row_gap: Val::Px(24.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Effect tails"),
                TextFont {
                    font: font.into(),
                    font_size: 36.0.into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(
                    "The same plop through the same reverb\n\
                     Only the time its track is kept alive differs.\n\n\
                     [1] no tail\n\
                     [2] 500 ms\n\
                     [3] 2 s (the default, outlasts the reverb)",
                ),
                normal.clone(),
                TextColor(Color::linear_rgb(0.7, 0.7, 0.7)),
            ));
            parent.spawn((
                Text::new("press [1], [2] or [3]"),
                normal,
                TextColor(COLOR_GONE),
                StatusText,
            ));
        });
}
