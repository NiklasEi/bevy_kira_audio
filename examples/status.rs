use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

#[derive(Resource)]
struct LoopAudioInstanceHandle(Handle<AudioInstance>);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, (start_audio, display_help_text))
        .add_systems(Update, (print_status, process_keyboard_input))
        .run();
}

fn start_audio(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let asset_handle = asset_server.load("sounds/loop.ogg");
    let instance_handle = audio.play(asset_handle).looped().handle();
    println!("Audio started.");
    commands.insert_resource(LoopAudioInstanceHandle(instance_handle));
}

fn process_keyboard_input(audio: Res<Audio>, kb: Res<ButtonInput<KeyCode>>) {
    if kb.just_pressed(KeyCode::KeyP) {
        audio.pause().linear_fade_out(Duration::from_millis(500));
        println!("Audio pausing...");
    } else if kb.just_pressed(KeyCode::KeyS) {
        audio.stop().fade_out(AudioTween::new(
            Duration::from_secs(1),
            AudioEasing::InOutPowi(2),
        ));
        println!("Audio stopping...");
    } else if kb.just_pressed(KeyCode::KeyR) {
        audio.resume().fade_in(AudioTween::new(
            Duration::from_millis(500),
            AudioEasing::InOutPowi(4),
        ));
        println!("Audio resuming...");
    }
}

fn print_status(audio: Res<Audio>, loop_audio: Res<LoopAudioInstanceHandle>) {
    // We could also get this info using the audio instance handle + asset (see instance_control example)
    // But: only the channel knows if the audio is currently queued. Using the method below,
    // we can differentiate between Queued and Stopped.
    let state = audio.state(&loop_audio.0);
    println!("Looping audio is {state:?}");
}

fn display_help_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let monogram = asset_server.load("fonts/monogram.ttf");
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: Color::rgba(0., 0., 0., 0.).into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "Press 'P' to pause\n".to_string(),
                            style: TextStyle {
                                font: monogram.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        },
                        TextSection {
                            value: "Press 'R' to resume\n".to_string(),
                            style: TextStyle {
                                font: monogram.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        },
                        TextSection {
                            value: "Press 'S' to stop\n\n".to_string(),
                            style: TextStyle {
                                font: monogram.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        },
                        TextSection {
                            value: "Check your console for the audio state".to_string(),
                            style: TextStyle {
                                font: monogram.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        },
                    ],
                    justify: JustifyText::Center,
                    ..Default::default()
                },
                ..Default::default()
            });
        });
}
