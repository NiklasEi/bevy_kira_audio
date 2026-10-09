use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

/// Reverb and delay keep sounding after the sound feeding them has ended. A sound with effects of
/// its own lets them ring out when it ends by itself, but cuts them off when it is stopped.
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, controls)
        .run();
}

#[derive(Resource)]
struct Sounds {
    plop: Handle<AudioSource>,
    music: Handle<AudioSource>,
}

#[derive(Resource, Default)]
struct Music(Option<Handle<AudioInstance>>);

fn reverb() -> ReverbBuilder {
    ReverbBuilder::new().feedback(0.9).damping(0.2).mix(0.7_f32)
}

fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
    mut music: ResMut<Music>,
    mut instances: ResMut<Assets<AudioInstance>>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        audio.play(sounds.plop.clone()).with_effect(reverb());
    }
    if keys.just_pressed(KeyCode::KeyM) && music.0.is_none() {
        music.0 = Some(
            audio
                .play(sounds.music.clone())
                .with_effect(reverb())
                .looped()
                .handle(),
        );
    }
    if keys.just_pressed(KeyCode::KeyS)
        && let Some(handle) = music.0.take()
        && let Some(mut instance) = instances.get_mut(&handle)
    {
        instance.stop(AudioTween::linear(Duration::from_millis(500)));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.insert_resource(Sounds {
        plop: asset_server.load("sounds/plop.ogg"),
        music: asset_server.load("sounds/loop.ogg"),
    });
    commands.init_resource::<Music>();

    commands.spawn((
        Node {
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        children![(
            Text::new(
                "Effect tails\n\n\
                 [P] play a plop: it ends by itself and its reverb rings out\n\
                 [M] play music through the same reverb\n\
                 [S] stop the music: its reverb fades out with it",
            ),
            TextFont {
                font: asset_server.load("fonts/monogram.ttf").into(),
                font_size: 26.0.into(),
                ..default()
            },
        )],
    ));
}
