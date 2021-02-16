use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource, ChannelId};
use std::collections::HashMap;

fn main() {
    let mut app = App::build();
    app.add_resource(Msaa { samples: 4 })
        .add_resource(WindowDescriptor {
            width: 800.,
            height: 600.,
            title: "Kira audio example".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .init_resource::<ButtonMaterials>()
        .add_startup_system(prepare_audio_and_ui.system())
        .add_system(check_audio_loading.system())
        .add_system(stop_button.system())
        .add_system(start_loop.system())
        .add_system(update_buttons.system())
        .add_system(play_pause_button.system());

    app.run();
}

fn check_audio_loading(mut audio_state: ResMut<AudioState>, asset_server: ResMut<AssetServer>) {
    if audio_state.audio_loaded {
        return;
    }
    if LoadState::Loaded == asset_server.get_load_state(&audio_state.loop_handle) {
        audio_state.audio_loaded = true;
    }
}

fn play_pause_button(
    button_materials: Res<ButtonMaterials>,
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &PlayPauseButton),
        Mutated<Interaction>,
    >,
    mut play_pause_text: Query<(&PlayPauseButtonText, &mut Text)>,
) {
    if !audio_state.audio_loaded {
        return;
    }
    for (interaction, mut material, button) in interaction_query.iter_mut() {
        let mut audio_state = audio_state.channels.get_mut(&button.channel).unwrap();
        if audio_state.stopped {
            *material = button_materials.disabled.clone();
            continue;
        }
        match *interaction {
            Interaction::Clicked => {
                if audio_state.paused {
                    audio.resume_channel(&button.channel);
                    for (text_button, mut text) in play_pause_text.iter_mut() {
                        if text_button.channel != button.channel {
                            continue;
                        }
                        text.value = "Pause".to_owned();
                    }
                } else {
                    audio.pause_channel(&button.channel);
                    for (text_button, mut text) in play_pause_text.iter_mut() {
                        if text_button.channel != button.channel {
                            continue;
                        }
                        text.value = "Play".to_owned();
                    }
                }
                audio_state.paused = !audio_state.paused;
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn stop_button(
    button_materials: Res<ButtonMaterials>,
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &StopButton),
        Mutated<Interaction>,
    >,
) {
    if !audio_state.audio_loaded {
        return;
    }
    for (interaction, mut material, button) in interaction_query.iter_mut() {
        let audio_state = audio_state.channels.get_mut(&button.channel).unwrap();
        if audio_state.stopped {
            *material = button_materials.disabled.clone();
            continue;
        }
        match *interaction {
            Interaction::Clicked => {
                audio.stop_channel(&button.channel);
                *audio_state = ChannelAudioState::default();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn start_loop(
    button_materials: Res<ButtonMaterials>,
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    asset_server: Res<AssetServer>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &StartLoopButton),
        Mutated<Interaction>,
    >,
) {
    if !audio_state.audio_loaded {
        return;
    }
    for (interaction, mut material, button) in interaction_query.iter_mut() {
        let mut audio_state = audio_state.channels.get_mut(&button.channel).unwrap();
        if audio_state.loop_started {
            *material = button_materials.disabled.clone();
            continue;
        }
        match *interaction {
            Interaction::Clicked => {
                let music: Handle<AudioSource> = asset_server.load("sounds/loop.ogg");
                audio.play_looped_in_channel(music.clone(), &button.channel);
                audio_state.loop_started = true;
                audio_state.stopped = false;
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn update_buttons(
    audio_state: ChangedRes<AudioState>,
    button_materials: Res<ButtonMaterials>,
    mut start_loop: Query<(&mut Handle<ColorMaterial>, &StartLoopButton)>,
    mut play_pause: Query<(&mut Handle<ColorMaterial>, &PlayPauseButton)>,
    mut stop: Query<(&mut Handle<ColorMaterial>, &StopButton)>,
    mut play_pause_text: Query<(&PlayPauseButtonText, &mut Text)>,
) {
    for (mut material, button) in start_loop.iter_mut() {
        *material = if audio_state.channels.get(&button.channel).unwrap().stopped {
            button_materials.normal.clone()
        } else {
            button_materials.disabled.clone()
        }
    }
    for (mut material, button) in play_pause.iter_mut() {
        let audio_state = audio_state.channels.get(&button.channel).unwrap();
        *material = if audio_state.stopped {
            button_materials.disabled.clone()
        } else {
            button_materials.normal.clone()
        };
        for (text_button, mut text) in play_pause_text.iter_mut() {
            if text_button.channel != button.channel {
                continue;
            }
            text.value = if audio_state.paused {
                "Play".to_owned()
            } else {
                "Pause".to_owned()
            };
        }
    }
    for (mut material, button) in stop.iter_mut() {
        *material = if audio_state.channels.get(&button.channel).unwrap().stopped {
            button_materials.disabled.clone()
        } else {
            button_materials.normal.clone()
        }
    }
}

struct PlayPauseButton {
    channel: ChannelId,
}

struct StartLoopButton {
    channel: ChannelId,
}

struct StopButton {
    channel: ChannelId,
}

struct PlayPauseButtonText {
    channel: ChannelId,
}

struct AudioState {
    audio_loaded: bool,
    loop_handle: Handle<AudioSource>,
    channels: HashMap<ChannelId, ChannelAudioState>,
}

struct ChannelAudioState {
    stopped: bool,
    paused: bool,
    loop_started: bool,
}

impl Default for ChannelAudioState {
    fn default() -> Self {
        ChannelAudioState {
            stopped: true,
            loop_started: false,
            paused: false,
        }
    }
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    disabled: Handle<ColorMaterial>,
}

impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            disabled: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
        }
    }
}

fn prepare_audio_and_ui(
    commands: &mut Commands,
    asset_server: ResMut<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    let mut channels = HashMap::new();
    channels.insert(
        ChannelId::new("first".to_owned()),
        ChannelAudioState::default(),
    );
    channels.insert(
        ChannelId::new("second".to_owned()),
        ChannelAudioState::default(),
    );
    channels.insert(
        ChannelId::new("third".to_owned()),
        ChannelAudioState::default(),
    );

    let loop_handle = asset_server.load("sounds/loop.ogg");
    let audio_state = AudioState {
        channels,
        audio_loaded: false,
        loop_handle,
    };

    set_up_ui(commands, asset_server, &audio_state, button_materials);

    commands.insert_resource(audio_state);
}

fn set_up_ui(
    commands: &mut Commands,
    asset_server: ResMut<AssetServer>,
    audio_state: &AudioState,
    button_materials: Res<ButtonMaterials>,
) {
    let font = asset_server.load("fonts/monogram.ttf");
    commands
        .spawn(CameraUiBundle::default())
        .spawn(NodeBundle {
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
                    .spawn(NodeBundle {
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
                            .spawn(NodeBundle {
                                style: Style {
                                    size: Size::new(Val::Percent(25.), Val::Percent(100.)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn(TextBundle {
                                    text: Text {
                                        value: format!("Channel {}", 3 - channel_index),
                                        font: font.clone(),
                                        style: TextStyle {
                                            font_size: 20.0,
                                            color: Color::rgb(0.2, 0.2, 0.2),
                                            ..Default::default()
                                        },
                                    },
                                    ..Default::default()
                                });
                            })
                            .spawn(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                                    margin: Rect::all(Val::Auto),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                material: button_materials.disabled.clone(),
                                ..Default::default()
                            })
                            .with(StartLoopButton {
                                channel: channel.clone(),
                            })
                            .with_children(|parent| {
                                parent.spawn(TextBundle {
                                    text: Text {
                                        value: "Start loop".to_string(),
                                        font: font.clone(),
                                        style: TextStyle {
                                            font_size: 20.0,
                                            color: Color::rgb(0.9, 0.9, 0.9),
                                            ..Default::default()
                                        },
                                    },
                                    ..Default::default()
                                });
                            })
                            .spawn(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                                    margin: Rect::all(Val::Auto),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                material: button_materials.disabled.clone(),
                                ..Default::default()
                            })
                            .with(PlayPauseButton {
                                channel: channel.clone(),
                            })
                            .with_children(|parent| {
                                parent
                                    .spawn(TextBundle {
                                        text: Text {
                                            value: "Pause".to_string(),
                                            font: font.clone(),
                                            style: TextStyle {
                                                font_size: 20.0,
                                                color: Color::rgb(0.9, 0.9, 0.9),
                                                ..Default::default()
                                            },
                                        },
                                        ..Default::default()
                                    })
                                    .with(PlayPauseButtonText {
                                        channel: channel.clone(),
                                    });
                            })
                            .spawn(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                                    margin: Rect::all(Val::Auto),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                material: button_materials.disabled.clone(),
                                ..Default::default()
                            })
                            .with(StopButton {
                                channel: channel.clone(),
                            })
                            .with_children(|parent| {
                                parent.spawn(TextBundle {
                                    text: Text {
                                        value: "Stop".to_string(),
                                        font: font.clone(),
                                        style: TextStyle {
                                            font_size: 20.0,
                                            color: Color::rgb(0.9, 0.9, 0.9),
                                            ..Default::default()
                                        },
                                    },
                                    ..Default::default()
                                });
                            });
                    });
            }
        });
}
