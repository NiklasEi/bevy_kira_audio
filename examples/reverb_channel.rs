use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use kira::effect::reverb::ReverbBuilder;
use std::time::Duration;

fn main() {
    let mut track = TrackBuilder::new();
    let _reverb = track.add_effect(
        ReverbBuilder::new()
            .feedback(0.85)
            .damping(0.2)
            .mix(0.5_f32),
    );

    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_audio_channel_with_track::<ReverbChannel>(track)
        .add_systems(Startup, start_playback)
        .add_systems(Update, play_next_sound)
        .run();
}

fn start_playback(
    channel: Res<AudioChannel<ReverbChannel>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    // Start with a looped sound immediately
    channel.play(asset_server.load("sounds/loop.ogg")).looped();

    // Queue the one-shot sounds to play in sequence
    commands.insert_resource(SoundQueue {
        sounds: vec![
            asset_server.load("sounds/cooking.ogg"),
            asset_server.load("sounds/plop.ogg"),
            asset_server.load("sounds/sound.ogg"),
        ],
        index: 0,
        timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
    });
}

fn play_next_sound(
    time: Res<Time>,
    channel: Res<AudioChannel<ReverbChannel>>,
    mut queue: ResMut<SoundQueue>,
) {
    queue.timer.tick(time.delta());
    if !queue.timer.just_finished() {
        return;
    }

    let sound = &queue.sounds[queue.index % queue.sounds.len()];
    channel.play(sound.clone());
    info!(
        "Playing sound {} with reverb",
        queue.index % queue.sounds.len()
    );
    queue.index += 1;
}

#[derive(Resource)]
struct ReverbChannel;

#[derive(Resource)]
struct SoundQueue {
    sounds: Vec<bevy::asset::Handle<AudioSource>>,
    index: usize,
    timer: Timer,
}
