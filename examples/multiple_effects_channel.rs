use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use kira::effect::distortion::{DistortionBuilder, DistortionHandle, DistortionKind};
use kira::effect::filter::{FilterBuilder, FilterHandle, FilterMode};
use kira::effect::reverb::{ReverbBuilder, ReverbHandle};
use std::time::Duration;

/// Interactive demo: three effects on one channel, toggled with keyboard keys.
///
/// Controls:
///   [1]              –  toggle looped sound on/off
///   [2] / [3] / [4]  –  play a one-shot sound
///   [F] / [D] / [R]  –  toggle Filter / Distortion / Reverb
fn main() {
    let mut track = TrackBuilder::new();

    let filter = track.add_effect(
        FilterBuilder::new()
            .mode(FilterMode::LowPass)
            .cutoff(800.0)
            .mix(0.0_f32),
    );

    let distortion = track.add_effect(
        DistortionBuilder::new()
            .kind(DistortionKind::SoftClip)
            .drive(Decibels(20.0))
            .mix(0.0_f32),
    );

    let reverb = track.add_effect(ReverbBuilder::new().feedback(0.8).damping(0.3).mix(0.0_f32));

    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_audio_channel_with_track::<EffectsChannel>(track)
        .insert_resource(EffectsState {
            filter,
            distortion,
            reverb,
            filter_on: false,
            distortion_on: false,
            reverb_on: false,
        })
        .insert_resource(LoopState {
            playing: false,
            instance: None,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, update_status_ui))
        .run();
}

#[derive(Resource)]
struct EffectsChannel;

#[derive(Resource)]
struct EffectsState {
    filter: FilterHandle,
    distortion: DistortionHandle,
    reverb: ReverbHandle,
    filter_on: bool,
    distortion_on: bool,
    reverb_on: bool,
}

#[derive(Resource)]
struct LoopState {
    playing: bool,
    instance: Option<Handle<AudioInstance>>,
}

#[derive(Resource)]
struct SoundHandles {
    loop_sound: Handle<AudioSource>,
    one_shots: [Handle<AudioSource>; 3],
}

#[derive(Component, Default)]
struct LoopStatusText;

#[derive(Component, Default)]
struct FilterStatusText;

#[derive(Component, Default)]
struct DistortionStatusText;

#[derive(Component, Default)]
struct ReverbStatusText;

const COLOR_ON: Color = Color::linear_rgb(0.2, 0.9, 0.2);
const COLOR_OFF: Color = Color::linear_rgb(0.45, 0.45, 0.45);
const COLOR_LABEL: Color = Color::linear_rgb(0.7, 0.7, 0.7);
const COLOR_KEY: Color = Color::linear_rgb(0.9, 0.8, 0.4);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let font = asset_server.load("fonts/monogram.ttf");
    commands.insert_resource(SoundHandles {
        loop_sound: asset_server.load("sounds/loop.ogg"),
        one_shots: [
            asset_server.load("sounds/cooking.ogg"),
            asset_server.load("sounds/plop.ogg"),
            asset_server.load("sounds/sound.ogg"),
        ],
    });

    let heading = TextFont {
        font: font.clone(),
        font_size: 36.0,
        ..default()
    };
    let normal = TextFont {
        font: font.clone(),
        font_size: 26.0,
        ..default()
    };

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(40.0)),
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|p| {
            text_line(p, "Multiple Effects Channel", &heading, Color::WHITE);
            spacer(p, 20.0);

            text_line(p, "Play sounds:", &normal, COLOR_LABEL);
            status_line::<LoopStatusText>(p, "  [1] loop     OFF", &normal);
            text_line(p, "  [2] cooking  [3] plop  [4] sound", &normal, COLOR_KEY);
            spacer(p, 20.0);

            text_line(p, "Toggle effects:", &normal, COLOR_LABEL);
            status_line::<FilterStatusText>(p, "  [F] Filter   OFF", &normal);
            status_line::<DistortionStatusText>(p, "  [D] Distort  OFF", &normal);
            status_line::<ReverbStatusText>(p, "  [R] Reverb   OFF", &normal);
        });
}

fn text_line(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    content: &str,
    font: &TextFont,
    color: Color,
) {
    parent.spawn((Text::new(content), font.clone(), TextColor(color)));
}

fn status_line<M: Component + Default>(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    content: &str,
    font: &TextFont,
) {
    parent.spawn((
        Text::new(content),
        font.clone(),
        TextColor(COLOR_OFF),
        M::default(),
    ));
}

fn spacer(parent: &mut RelatedSpawnerCommands<ChildOf>, height: f32) {
    parent.spawn(Node {
        height: Val::Px(height),
        ..default()
    });
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    channel: Res<AudioChannel<EffectsChannel>>,
    sounds: Res<SoundHandles>,
    mut effects: ResMut<EffectsState>,
    mut loop_state: ResMut<LoopState>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    let tween = kira::Tween {
        duration: Duration::from_millis(100),
        ..default()
    };

    // Toggle loop
    if keys.just_pressed(KeyCode::Digit1) {
        if loop_state.playing {
            if let Some(ref handle) = loop_state.instance
                && let Some(instance) = audio_instances.get_mut(handle)
            {
                instance.stop(AudioTween::default());
            }
            loop_state.instance = None;
            loop_state.playing = false;
        } else {
            let handle = channel.play(sounds.loop_sound.clone()).looped().handle();
            loop_state.instance = Some(handle);
            loop_state.playing = true;
        }
    }

    // One-shot sounds
    if keys.just_pressed(KeyCode::Digit2) {
        channel.play(sounds.one_shots[0].clone());
    }
    if keys.just_pressed(KeyCode::Digit3) {
        channel.play(sounds.one_shots[1].clone());
    }
    if keys.just_pressed(KeyCode::Digit4) {
        channel.play(sounds.one_shots[2].clone());
    }

    // Toggle effects
    if keys.just_pressed(KeyCode::KeyF) {
        effects.filter_on = !effects.filter_on;
        let mix = if effects.filter_on { 1.0_f32 } else { 0.0_f32 };
        effects.filter.set_mix(mix, tween);
    }
    if keys.just_pressed(KeyCode::KeyD) {
        effects.distortion_on = !effects.distortion_on;
        let mix = if effects.distortion_on {
            1.0_f32
        } else {
            0.0_f32
        };
        effects.distortion.set_mix(mix, tween);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        effects.reverb_on = !effects.reverb_on;
        let mix = if effects.reverb_on { 0.5_f32 } else { 0.0_f32 };
        effects.reverb.set_mix(mix, tween);
    }
}

#[allow(clippy::type_complexity)]
fn update_status_ui(
    effects: Res<EffectsState>,
    loop_state: Res<LoopState>,
    mut loop_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<LoopStatusText>,
            Without<FilterStatusText>,
            Without<DistortionStatusText>,
            Without<ReverbStatusText>,
        ),
    >,
    mut filter_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<FilterStatusText>,
            Without<LoopStatusText>,
            Without<DistortionStatusText>,
            Without<ReverbStatusText>,
        ),
    >,
    mut distortion_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<DistortionStatusText>,
            Without<LoopStatusText>,
            Without<FilterStatusText>,
            Without<ReverbStatusText>,
        ),
    >,
    mut reverb_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<ReverbStatusText>,
            Without<LoopStatusText>,
            Without<FilterStatusText>,
            Without<DistortionStatusText>,
        ),
    >,
) {
    if loop_state.is_changed() {
        update_line(&mut loop_q, "  [1] loop  ", loop_state.playing);
    }
    if effects.is_changed() {
        update_line(&mut filter_q, "  [F] Filter", effects.filter_on);
        update_line(&mut distortion_q, "  [D] Distort", effects.distortion_on);
        update_line(&mut reverb_q, "  [R] Reverb", effects.reverb_on);
    }
}

fn update_line(
    query: &mut Query<(&mut Text, &mut TextColor), impl bevy::ecs::query::QueryFilter>,
    prefix: &str,
    on: bool,
) {
    if let Ok((mut text, mut color)) = query.single_mut() {
        let label = if on { "ON" } else { "OFF" };
        *text = Text::new(format!("{}   {}", prefix, label));
        color.0 = if on { COLOR_ON } else { COLOR_OFF };
    }
}
