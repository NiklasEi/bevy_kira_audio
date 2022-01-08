#![allow(clippy::type_complexity)]

use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};
use std::collections::HashMap;

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        width: 800.,
        height: 600.,
        title: "Kira audio example".to_string(),
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(AudioPlugin)
    .add_startup_system(prepare_audio_and_ui.system())
    .add_system(check_audio_loading.system())
    .add_system(stop_button.system())
    .add_system(start_loop.system())
    .add_system(update_start_loop_buttons.system())
    .add_system(update_play_pause_buttons.system())
    .add_system(update_play_single_sound_buttons.system())
    .add_system(update_stop_buttons.system())
    .add_system(update_volume_buttons.system())
    .add_system(control_volume.system())
    .add_system(play_single_sound.system())
    .add_system(play_pause_button.system());

    app.run();
}

fn check_audio_loading(mut audio_state: ResMut<AudioState>, asset_server: ResMut<AssetServer>) {
    if audio_state.audio_loaded
        || LoadState::Loaded != asset_server.get_load_state(&audio_state.loop_handle)
        || LoadState::Loaded != asset_server.get_load_state(&audio_state.sound_handle)
    {
        return;
    }
    audio_state.audio_loaded = true;
}

fn play_pause_button(
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut interaction_query: Query<
        (&Interaction, &Channel),
        (Changed<Interaction>, With<PlayPauseButton>),
    >,
) {
    if !audio_state.audio_loaded {
        return;
    }
    for (interaction, button) in interaction_query.iter_mut() {
        let mut audio_state = audio_state.channels.get_mut(&button.channel).unwrap();
        if audio_state.stopped {
            continue;
        }
        if interaction == &Interaction::Clicked {
            if audio_state.paused {
                audio.resume_channel(&button.channel);
            } else {
                audio.pause_channel(&button.channel);
            }
            audio_state.paused = !audio_state.paused;
        }
    }
}

fn stop_button(
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut interaction_query: Query<
        (&Interaction, &Channel),
        (Changed<Interaction>, With<StopButton>),
    >,
) {
    if !audio_state.audio_loaded {
        return;
    }
    for (interaction, button) in interaction_query.iter_mut() {
        let audio_state = audio_state.channels.get_mut(&button.channel).unwrap();
        if audio_state.stopped {
            continue;
        }
        if interaction == &Interaction::Clicked {
            audio.stop_channel(&button.channel);
            *audio_state = ChannelAudioState::default();
        }
    }
}

fn start_loop(
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut interaction_query: Query<
        (&Interaction, &Channel),
        (Changed<Interaction>, With<StartLoopButton>),
    >,
) {
    if !audio_state.audio_loaded {
        return;
    }
    for (interaction, button) in interaction_query.iter_mut() {
        let mut channel_audio_state = audio_state.channels.get_mut(&button.channel).unwrap();
        if channel_audio_state.loop_started {
            continue;
        }
        if interaction == &Interaction::Clicked {
            channel_audio_state.loop_started = true;
            channel_audio_state.stopped = false;
            audio.play_looped_in_channel(audio_state.loop_handle.clone(), &button.channel);
        }
    }
}

fn play_single_sound(
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut interaction_query: Query<
        (&Interaction, &Channel),
        (Changed<Interaction>, With<PlaySingleSound>),
    >,
) {
    if !audio_state.audio_loaded {
        return;
    }
    for (interaction, button) in interaction_query.iter_mut() {
        let mut channel_audio_state = audio_state.channels.get_mut(&button.channel).unwrap();
        if interaction == &Interaction::Clicked {
            channel_audio_state.paused = false;
            channel_audio_state.stopped = false;
            audio.play_in_channel(audio_state.sound_handle.clone(), &button.channel);
        }
    }
}

fn control_volume(
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut interaction_query: Query<
        (&Interaction, &Channel, &ChangeVolumeButton),
        Changed<Interaction>,
    >,
) {
    for (interaction, channel, volume) in interaction_query.iter_mut() {
        let mut channel_audio_state = audio_state.channels.get_mut(&channel.channel).unwrap();
        if interaction == &Interaction::Clicked {
            if volume.louder {
                channel_audio_state.volume += 0.1;
            } else {
                channel_audio_state.volume -= 0.1;
            }
            audio.set_volume_in_channel(channel_audio_state.volume, &channel.channel);
        }
    }
}

fn update_start_loop_buttons(
    audio_state: Res<AudioState>,
    mut start_loop: Query<(&Interaction, &mut UiColor, &Channel), With<StartLoopButton>>,
) {
    for (interaction, mut color, button) in start_loop.iter_mut() {
        *color = if !audio_state
            .channels
            .get(&button.channel)
            .unwrap()
            .loop_started
            && audio_state.audio_loaded
        {
            if interaction == &Interaction::Hovered {
                HOVERED_BUTTON.into()
            } else {
                NORMAL_BUTTON.into()
            }
        } else {
            DISABLED_BUTTON.into()
        }
    }
}

fn update_play_pause_buttons(
    audio_state: Res<AudioState>,
    mut play_pause: Query<(&Interaction, &mut UiColor, &Channel), With<PlayPauseButton>>,
    mut play_pause_text: Query<(&Channel, &mut Text)>,
) {
    for (interaction, mut color, button) in play_pause.iter_mut() {
        let audio_state = audio_state.channels.get(&button.channel).unwrap();
        *color = if audio_state.stopped {
            DISABLED_BUTTON.into()
        } else if interaction == &Interaction::Hovered {
            HOVERED_BUTTON.into()
        } else {
            NORMAL_BUTTON.into()
        };
        for (text_button, mut text) in play_pause_text.iter_mut() {
            if text_button.channel == button.channel {
                text.sections.first_mut().unwrap().value = if audio_state.paused {
                    "Play".to_owned()
                } else {
                    "Pause".to_owned()
                };
            }
        }
    }
}

fn update_play_single_sound_buttons(
    audio_state: Res<AudioState>,
    mut play_single_sound: Query<(&Interaction, &mut UiColor, &Channel), With<PlaySingleSound>>,
) {
    for (interaction, mut material, _button) in play_single_sound.iter_mut() {
        *material = if audio_state.audio_loaded {
            if interaction == &Interaction::Hovered {
                HOVERED_BUTTON.into()
            } else {
                NORMAL_BUTTON.into()
            }
        } else {
            DISABLED_BUTTON.into()
        }
    }
}

fn update_stop_buttons(
    audio_state: Res<AudioState>,
    mut stop: Query<(&Interaction, &mut UiColor, &Channel), With<StopButton>>,
) {
    for (interaction, mut material, button) in stop.iter_mut() {
        *material = if audio_state.channels.get(&button.channel).unwrap().stopped {
            DISABLED_BUTTON.into()
        } else if interaction == &Interaction::Hovered {
            HOVERED_BUTTON.into()
        } else {
            NORMAL_BUTTON.into()
        }
    }
}

fn update_volume_buttons(
    mut volume: Query<(&Interaction, &mut UiColor), With<ChangeVolumeButton>>,
) {
    for (interaction, mut material) in volume.iter_mut() {
        *material = if interaction == &Interaction::Hovered {
            HOVERED_BUTTON.into()
        } else {
            NORMAL_BUTTON.into()
        }
    }
}

#[derive(Component)]
struct PlayPauseButton;

#[derive(Component)]
struct PlaySingleSound;

#[derive(Component)]
struct StartLoopButton;

#[derive(Component)]
struct ChangeVolumeButton {
    louder: bool,
}

#[derive(Component)]
struct StopButton;

#[derive(Component)]
struct Channel {
    channel: AudioChannel,
}

struct AudioState {
    audio_loaded: bool,
    loop_handle: Handle<AudioSource>,
    sound_handle: Handle<AudioSource>,
    channels: HashMap<AudioChannel, ChannelAudioState>,
}

struct ChannelAudioState {
    stopped: bool,
    paused: bool,
    loop_started: bool,
    volume: f32,
}

impl Default for ChannelAudioState {
    fn default() -> Self {
        ChannelAudioState {
            volume: 1.0,
            stopped: true,
            loop_started: false,
            paused: false,
        }
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const DISABLED_BUTTON: Color = Color::rgb(0.5, 0.5, 0.5);

fn prepare_audio_and_ui(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let mut channels = HashMap::new();
    channels.insert(
        AudioChannel::new("first".to_owned()),
        ChannelAudioState::default(),
    );
    channels.insert(
        AudioChannel::new("second".to_owned()),
        ChannelAudioState::default(),
    );
    channels.insert(
        AudioChannel::new("third".to_owned()),
        ChannelAudioState::default(),
    );

    let loop_handle = asset_server.load("sounds/loop.ogg");
    let sound_handle = asset_server.load("sounds/sound.ogg");
    let audio_state = AudioState {
        channels,
        audio_loaded: false,
        loop_handle,
        sound_handle,
    };

    set_up_ui(&mut commands, asset_server, &audio_state);

    commands.insert_resource(audio_state);
}

fn set_up_ui(commands: &mut Commands, asset_server: ResMut<AssetServer>, audio_state: &AudioState) {
    let font = asset_server.load("fonts/monogram.ttf");
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            for (channel_index, (channel, _state)) in audio_state.channels.iter().enumerate() {
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            size: Size::new(Val::Percent(100.), Val::Percent(33.3)),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    size: Size::new(Val::Px(120.0), Val::Percent(100.)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn_bundle(TextBundle {
                                    text: Text {
                                        sections: vec![TextSection {
                                            value: format!("Channel {}", 3 - channel_index),
                                            style: TextStyle {
                                                font_size: 20.0,
                                                color: Color::rgb(0.2, 0.2, 0.2),
                                                font: font.clone(),
                                            },
                                        }],
                                        alignment: Default::default(),
                                    },
                                    ..Default::default()
                                });
                            });
                        spawn_button(
                            parent,
                            channel,
                            "Sound",
                            DISABLED_BUTTON.into(),
                            PlaySingleSound,
                            font.clone(),
                        );
                        spawn_button(
                            parent,
                            channel,
                            "Loop",
                            DISABLED_BUTTON.into(),
                            StartLoopButton,
                            font.clone(),
                        );
                        parent
                            .spawn_bundle(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(100.0), Val::Px(65.0)),
                                    margin: Rect::all(Val::Auto),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                color: DISABLED_BUTTON.into(),
                                ..Default::default()
                            })
                            .insert(PlayPauseButton)
                            .insert(Channel {
                                channel: channel.clone(),
                            })
                            .with_children(|parent| {
                                parent
                                    .spawn_bundle(TextBundle {
                                        text: Text {
                                            sections: vec![TextSection {
                                                value: "Pause".to_owned(),
                                                style: TextStyle {
                                                    font_size: 20.0,
                                                    color: Color::rgb(0.9, 0.9, 0.9),
                                                    font: font.clone(),
                                                },
                                            }],
                                            alignment: Default::default(),
                                        },
                                        ..Default::default()
                                    })
                                    .insert(Channel {
                                        channel: channel.clone(),
                                    });
                            });
                        spawn_button(
                            parent,
                            channel,
                            "Vol. up",
                            NORMAL_BUTTON.into(),
                            ChangeVolumeButton { louder: true },
                            font.clone(),
                        );
                        spawn_button(
                            parent,
                            channel,
                            "Vol. down",
                            NORMAL_BUTTON.into(),
                            ChangeVolumeButton { louder: false },
                            font.clone(),
                        );
                        spawn_button(
                            parent,
                            channel,
                            "Stop",
                            DISABLED_BUTTON.into(),
                            StopButton,
                            font.clone(),
                        );
                    });
            }
        });
}

fn spawn_button<T: Component>(
    parent: &mut ChildBuilder,
    channel: &AudioChannel,
    text: &str,
    color: UiColor,
    marker: T,
    font: Handle<Font>,
) {
    parent
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(100.0), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color,
            ..Default::default()
        })
        .insert(marker)
        .insert(Channel {
            channel: channel.clone(),
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: text.to_string(),
                        style: TextStyle {
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            font: font.clone(),
                        },
                    }],
                    alignment: Default::default(),
                },
                ..Default::default()
            });
        });
}
