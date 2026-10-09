use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

fn main() {
    // The reverb applies to every sound played on the hall channel, including sounds that bring
    // effects of their own.
    let hall =
        AudioTrack::new().with_effect(ReverbBuilder::new().feedback(0.9).damping(0.1).mix(0.6_f32));

    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_audio_channel_with_track::<HallChannel>(hall)
        .init_resource::<Playing>()
        .add_systems(Startup, setup)
        .add_systems(Update, (play, update_status).chain())
        .run();
}

/// One way of playing the loop: through the hall channel, through a low-pass, or both.
struct Variant {
    key: KeyCode,
    /// Play on the channel carrying the reverb rather than on the main channel.
    hall: bool,
    /// Give the sound a low-pass filter of its own.
    muffle: bool,
    label: &'static str,
    routing: &'static str,
}

const VARIANTS: [Variant; 4] = [
    Variant {
        key: KeyCode::Digit1,
        hall: false,
        muffle: false,
        label: "[1] dry",
        routing: "loop -> main track",
    },
    Variant {
        key: KeyCode::Digit2,
        hall: false,
        muffle: true,
        label: "[2] muffled",
        routing: "loop -> low-pass -> main track",
    },
    Variant {
        key: KeyCode::Digit3,
        hall: true,
        muffle: false,
        label: "[3] hall",
        routing: "loop -> reverb -> main track",
    },
    Variant {
        key: KeyCode::Digit4,
        hall: true,
        muffle: true,
        label: "[4] both",
        routing: "loop -> low-pass -> reverb -> main track",
    },
];

const COLOR_PLAYING: Color = Color::linear_rgb(0.3, 0.85, 0.4);
const COLOR_IDLE: Color = Color::linear_rgb(0.45, 0.45, 0.45);

#[derive(Resource)]
struct Loop(Handle<AudioSource>);

/// The variant that is currently looping, if any.
#[derive(Resource, Default)]
struct Playing(Option<&'static Variant>);

#[derive(Component)]
struct StatusText;

fn play(
    keys: Res<ButtonInput<KeyCode>>,
    audio: Res<Audio>,
    hall: Res<AudioChannel<HallChannel>>,
    sound: Res<Loop>,
    mut playing: ResMut<Playing>,
) {
    for variant in &VARIANTS {
        if !keys.just_pressed(variant.key) {
            continue;
        }

        audio.stop();
        hall.stop();

        // One statement per variant, each playing the same loop. `with_effect` gives the sound an
        // effect of its own; playing on the hall channel adds that channel's reverb on top.
        match (variant.hall, variant.muffle) {
            // Dry: no effects at all.
            (false, false) => {
                audio.play(sound.0.clone()).looped();
            }
            // The sound's own effect only.
            (false, true) => {
                audio.play(sound.0.clone()).with_effect(low_pass()).looped();
            }
            // The channel's effect only.
            (true, false) => {
                hall.play(sound.0.clone()).looped();
            }
            // Both: the sound's own effect first, then the channel's.
            (true, true) => {
                hall.play(sound.0.clone()).with_effect(low_pass()).looped();
            }
        }

        playing.0 = Some(variant);
        info!("Playing {}: {}", variant.label, variant.routing);
    }
}

/// The effect a sound brings itself: a low-pass leaving only the bass.
fn low_pass() -> FilterBuilder {
    FilterBuilder::new()
        .mode(FilterMode::LowPass)
        .cutoff(400.0)
        .mix(1.0_f32)
}

fn update_status(
    playing: Res<Playing>,
    mut status: Single<(&mut Text, &mut TextColor), With<StatusText>>,
) {
    if !playing.is_changed() {
        return;
    }

    let (text, color) = &mut *status;
    match playing.0 {
        Some(variant) => {
            **text = Text::new(format!("{}  -  {}", variant.label, variant.routing));
            color.0 = COLOR_PLAYING;
        }
        None => {
            **text = Text::new("press [1], [2], [3] or [4]");
            color.0 = COLOR_IDLE;
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.insert_resource(Loop(asset_server.load("sounds/loop.ogg")));

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
                Text::new("Stacked effects"),
                TextFont {
                    font: font.into(),
                    font_size: 36.0.into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(
                    "The same loop through a per-sound low-pass, a channel reverb, or both.\n\n\
                     [1] dry\n\
                     [2] muffled  -  the sound's own low-pass\n\
                     [3] hall     -  the channel's reverb\n\
                     [4] both     -  low-pass first, then the same reverb",
                ),
                normal.clone(),
                TextColor(Color::linear_rgb(0.7, 0.7, 0.7)),
            ));
            parent.spawn((
                Text::new("press [1], [2], [3] or [4]"),
                normal,
                TextColor(COLOR_IDLE),
                StatusText,
            ));
        });
}

#[derive(Resource)]
struct HallChannel;
