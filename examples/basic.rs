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

fn update_buttons(
    audio_state: Res<AudioState>,
    button_materials: Res<ButtonMaterials>,
    mut start_loop: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Channel),
        With<StartLoopButton>,
    >,
    mut play_pause: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Channel),
        With<PlayPauseButton>,
    >,
    mut play_single_sound: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Channel),
        With<PlaySingleSound>,
    >,
    mut stop: Query<(&Interaction, &mut Handle<ColorMaterial>, &Channel), With<StopButton>>,
    mut play_pause_text: Query<(&Channel, &mut Text)>,
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
                text.value = if audio_state.paused {
                    "Play".to_owned()
                } else {
                    "Pause".to_owned()
                };
            }
        }
    }
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

struct PlayPauseButton;

struct PlaySingleSound;

struct StartLoopButton;

struct StopButton;

struct Channel {
    channel: ChannelId,
}

struct AudioState {
    audio_loaded: bool,
    loop_handle: Handle<AudioSource>,
    sound_handle: Handle<AudioSource>,
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
    let sound_handle = asset_server.load("sounds/sound.ogg");
    let audio_state = AudioState {
        channels,
        audio_loaded: false,
        loop_handle,
        sound_handle,
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
                            });
                        spawn_button(
                            parent,
                            channel,
                            "Play sound",
                            &button_materials,
                            PlaySingleSound,
                            font.clone(),
                        );
                        spawn_button(
                            parent,
                            channel,
                            "Start loop",
                            &button_materials,
                            StartLoopButton,
                            font.clone(),
                        );
                        spawn_button(
                            parent,
                            channel,
                            "Pause",
                            &button_materials,
                            PlayPauseButton,
                            font.clone(),
                        );
                        spawn_button(
                            parent,
                            channel,
                            "Stop",
                            &button_materials,
                            StopButton,
                            font.clone(),
                        );
                    });
            }
        });
}

fn spawn_button<T: 'static + Send + Sync>(
    parent: &mut ChildBuilder,
    channel: &ChannelId,
    text: &str,
    button_materials: &ButtonMaterials,
    marker: T,
    font: Handle<Font>,
) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.disabled.clone(),
            ..Default::default()
        })
        .with(marker)
        .with(Channel {
            channel: channel.clone(),
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    value: text.to_string(),
                    font,
                    style: TextStyle {
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        });
}
