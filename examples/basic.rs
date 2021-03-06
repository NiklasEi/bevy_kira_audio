use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};
use std::collections::HashMap;

fn main() {
    let mut app = App::build();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
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
        (Mutated<Interaction>, With<PlayPauseButton>),
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
        (Mutated<Interaction>, With<StopButton>),
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
        (Mutated<Interaction>, With<StartLoopButton>),
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
        (Mutated<Interaction>, With<PlaySingleSound>),
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
        Mutated<Interaction>,
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
    button_materials: Res<ButtonMaterials>,
    mut start_loop: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Channel),
        With<StartLoopButton>,
    >,
) {
    for (interaction, mut material, button) in start_loop.iter_mut() {
        *material = if !audio_state
            .channels
            .get(&button.channel)
            .unwrap()
            .loop_started
            && audio_state.audio_loaded
        {
            if interaction == &Interaction::Hovered {
                button_materials.hovered.clone()
            } else {
                button_materials.normal.clone()
            }
        } else {
            button_materials.disabled.clone()
        }
    }
}

fn update_play_pause_buttons(
    audio_state: Res<AudioState>,
    button_materials: Res<ButtonMaterials>,
    mut play_pause: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Channel),
        With<PlayPauseButton>,
    >,
    mut play_pause_text: Query<(&Channel, &mut Text)>,
) {
    for (interaction, mut material, button) in play_pause.iter_mut() {
        let audio_state = audio_state.channels.get(&button.channel).unwrap();
        *material = if audio_state.stopped {
            button_materials.disabled.clone()
        } else {
            if interaction == &Interaction::Hovered {
                button_materials.hovered.clone()
            } else {
                button_materials.normal.clone()
            }
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
    button_materials: Res<ButtonMaterials>,
    mut play_single_sound: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Channel),
        With<PlaySingleSound>,
    >,
) {
    for (interaction, mut material, _button) in play_single_sound.iter_mut() {
        *material = if audio_state.audio_loaded {
            if interaction == &Interaction::Hovered {
                button_materials.hovered.clone()
            } else {
                button_materials.normal.clone()
            }
        } else {
            button_materials.disabled.clone()
        }
    }
}

fn update_stop_buttons(
    audio_state: Res<AudioState>,
    button_materials: Res<ButtonMaterials>,
    mut stop: Query<(&Interaction, &mut Handle<ColorMaterial>, &Channel), With<StopButton>>,
) {
    for (interaction, mut material, button) in stop.iter_mut() {
        *material = if audio_state.channels.get(&button.channel).unwrap().stopped {
            button_materials.disabled.clone()
        } else {
            if interaction == &Interaction::Hovered {
                button_materials.hovered.clone()
            } else {
                button_materials.normal.clone()
            }
        }
    }
}

fn update_volume_buttons(
    mut volume: Query<(&Interaction, &mut Handle<ColorMaterial>), With<ChangeVolumeButton>>,
) {
    for (interaction, mut material) in volume.iter_mut() {
        *material = if interaction == &Interaction::Hovered {
            button_materials.hovered.clone()
        } else {
            button_materials.normal.clone()
        }
    }
}

struct PlayPauseButton;

struct PlaySingleSound;

struct StartLoopButton;

struct ChangeVolumeButton {
    louder: bool,
}

struct StopButton;

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

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    disabled: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            disabled: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
        }
    }
}

fn prepare_audio_and_ui(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
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

    set_up_ui(&mut commands, asset_server, &audio_state, button_materials);

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
        .spawn(UiCameraBundle::default())
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
                                    size: Size::new(Val::Px(120.0), Val::Percent(100.)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn(TextBundle {
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
                            button_materials.disabled.clone(),
                            PlaySingleSound,
                            font.clone(),
                        );
                        spawn_button(
                            parent,
                            channel,
                            "Loop",
                            button_materials.disabled.clone(),
                            StartLoopButton,
                            font.clone(),
                        );
                        parent
                            .spawn(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(100.0), Val::Px(65.0)),
                                    margin: Rect::all(Val::Auto),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                material: button_materials.disabled.clone(),
                                ..Default::default()
                            })
                            .with(PlayPauseButton)
                            .with(Channel {
                                channel: channel.clone(),
                            })
                            .with_children(|parent| {
                                parent
                                    .spawn(TextBundle {
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
                                    .with(Channel {
                                        channel: channel.clone(),
                                    });
                            });
                        spawn_button(
                            parent,
                            channel,
                            "Vol. up",
                            button_materials.normal.clone(),
                            ChangeVolumeButton { louder: true },
                            font.clone(),
                        );
                        spawn_button(
                            parent,
                            channel,
                            "Vol. down",
                            button_materials.normal.clone(),
                            ChangeVolumeButton { louder: false },
                            font.clone(),
                        );
                        spawn_button(
                            parent,
                            channel,
                            "Stop",
                            button_materials.disabled.clone(),
                            StopButton,
                            font.clone(),
                        );
                    });
            }
        });
}

fn spawn_button<T: 'static + Send + Sync>(
    parent: &mut ChildBuilder,
    channel: &AudioChannel,
    text: &str,
    material: Handle<ColorMaterial>,
    marker: T,
    font: Handle<Font>,
) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(100.0), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material,
            ..Default::default()
        })
        .with(marker)
        .with(Channel {
            channel: channel.clone(),
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
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
