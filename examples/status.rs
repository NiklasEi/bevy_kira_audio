use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

struct LoopAudioInstanceHandle {
    instance_handle: Handle<AudioInstance>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(start_audio)
        .add_startup_system(display_help_text)
        .add_system(print_status)
        .add_system(process_keyboard_input)
        .run();
}

fn start_audio(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let asset_handle = asset_server.load("sounds/loop.ogg");
    let instance_handle = audio.play(asset_handle).looped().handle();
    println!("Audio started.");
    commands.insert_resource(LoopAudioInstanceHandle { instance_handle });
}

fn process_keyboard_input(audio: Res<Audio>, kb: Res<Input<KeyCode>>) {
    if kb.just_pressed(KeyCode::P) {
        audio.pause().linear_fade_out(Duration::from_millis(500));
        println!("Audio pausing...");
    } else if kb.just_pressed(KeyCode::S) {
        audio.stop().fade_out(AudioTween::new(
            Duration::from_secs(1),
            AudioEasing::InOutPowi(2),
        ));
        println!("Audio stopping...");
    } else if kb.just_pressed(KeyCode::R) {
        audio.resume().fade_in(AudioTween::new(
            Duration::from_millis(500),
            AudioEasing::InOutPowi(4),
        ));
        println!("Audio resuming...");
    }
}

fn print_status(audio: Res<Audio>, loop_audio: Res<LoopAudioInstanceHandle>) {
    let state = audio.state(&loop_audio.instance_handle);
    println!("Looping audio is {:?}", state);
}

fn display_help_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let monogram = asset_server.load("fonts/monogram.ttf");
    commands.spawn_bundle(Camera2dBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: Color::rgba(0., 0., 0., 0.).into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
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
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                },
                ..Default::default()
            });
        });
}
