use bevy::ecs::schedule::SystemConfigs;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_kira_audio::AudioApp;
use std::clone::Clone;
use std::marker::PhantomData;

// This is a bigger example with a GUI for full control over three audio channels
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .init_resource::<LastAction>()
        .add_systems(Startup, prepare_audio_and_ui)
        .add_systems(Update, create_row_systems::<FirstChannel>())
        .add_systems(Update, create_row_systems::<SecondChannel>())
        .add_systems(Update, create_row_systems::<ThirdChannel>())
        .add_audio_channel::<FirstChannel>()
        .add_audio_channel::<SecondChannel>()
        .add_audio_channel::<ThirdChannel>()
        .run();
}

fn create_row_systems<T: Component + Default>() -> SystemConfigs {
    (
        stop_button::<T>,
        loop_button::<T>,
        volume_buttons::<T>,
        play_sound_button::<T>,
        play_pause_button::<T>,
    )
        .into_configs()
}

fn play_pause_button<T: Component + Default>(
    channel: Res<AudioChannel<T>>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
    time: Res<Time>,
    mut last_action: ResMut<LastAction>,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), With<PlayPauseButton<T>>>,
    mut play_pause_text: Query<&mut Text, With<PlayPauseButton<T>>>,
) {
    let (interaction, mut color) = interaction_query.single_mut();
    *color = if channel_state.stopped {
        DISABLED_BUTTON.into()
    } else if interaction == &Interaction::Hovered {
        HOVERED_BUTTON.into()
    } else {
        NORMAL_BUTTON.into()
    };
    let mut text = play_pause_text.single_mut();
    text.sections.first_mut().unwrap().value = if channel_state.paused {
        "Play".to_owned()
    } else {
        "Pause".to_owned()
    };
    if channel_state.stopped {
        return;
    }
    if interaction == &Interaction::Pressed {
        if !last_action.action(&time) {
            return;
        }
        if channel_state.paused {
            channel.resume();
        } else {
            channel.pause();
        }
        channel_state.paused = !channel_state.paused;
    }
}

fn stop_button<T: Component + Default>(
    channel: Res<AudioChannel<T>>,
    time: Res<Time>,
    mut last_action: ResMut<LastAction>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), With<StopButton<T>>>,
) {
    let (interaction, mut color) = interaction_query.single_mut();
    *color = if channel_state.stopped {
        DISABLED_BUTTON.into()
    } else if interaction == &Interaction::Hovered {
        HOVERED_BUTTON.into()
    } else {
        NORMAL_BUTTON.into()
    };
    if channel_state.stopped {
        return;
    }
    if interaction == &Interaction::Pressed {
        if !last_action.action(&time) {
            return;
        }
        channel.stop();
        *channel_state = ChannelAudioState::<T>::default();
    }
}

fn loop_button<T: Component + Default>(
    channel: Res<AudioChannel<T>>,
    time: Res<Time>,
    mut last_action: ResMut<LastAction>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
    audio_handles: Res<AudioHandles>,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), With<StartLoopButton<T>>>,
) {
    let (interaction, mut color) = interaction_query.single_mut();
    *color = if !channel_state.loop_started {
        if interaction == &Interaction::Hovered {
            HOVERED_BUTTON.into()
        } else {
            NORMAL_BUTTON.into()
        }
    } else {
        DISABLED_BUTTON.into()
    };
    if channel_state.loop_started {
        return;
    }
    if interaction == &Interaction::Pressed {
        if !last_action.action(&time) {
            return;
        }
        channel_state.loop_started = true;
        channel_state.stopped = false;
        channel.play(audio_handles.loop_handle.clone()).looped();
    }
}

fn play_sound_button<T: Component + Default>(
    channel: Res<AudioChannel<T>>,
    time: Res<Time>,
    mut last_action: ResMut<LastAction>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
    audio_handles: Res<AudioHandles>,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), With<PlaySoundButton<T>>>,
) {
    let (interaction, mut color) = interaction_query.single_mut();
    *color = if interaction == &Interaction::Hovered {
        HOVERED_BUTTON.into()
    } else {
        NORMAL_BUTTON.into()
    };
    if interaction == &Interaction::Pressed {
        if !last_action.action(&time) {
            return;
        }
        channel_state.paused = false;
        channel_state.stopped = false;
        channel.play(audio_handles.sound_handle.clone());
    }
}

fn volume_buttons<T: Component + Default>(
    channel: Res<AudioChannel<T>>,
    time: Res<Time>,
    mut last_action: ResMut<LastAction>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &ChangeVolumeButton<T>)>,
) {
    for (interaction, mut color, volume) in &mut interaction_query {
        *color = if interaction == &Interaction::Hovered {
            HOVERED_BUTTON.into()
        } else {
            NORMAL_BUTTON.into()
        };
        if interaction == &Interaction::Pressed {
            if !last_action.action(&time) {
                return;
            }
            if volume.louder {
                channel_state.volume += 0.1;
            } else {
                channel_state.volume = (channel_state.volume - 0.1).max(0.);
            }
            channel.set_volume(channel_state.volume);
        }
    }
}

#[derive(Resource, Default)]
struct LastAction(f64);

impl LastAction {
    fn action(&mut self, time: &Time) -> bool {
        if time.elapsed_seconds_f64() - self.0 < 0.2 {
            return false;
        }
        self.0 = time.elapsed_seconds_f64();

        true
    }
}

#[derive(Component, Default, Clone)]
struct PlayPauseButton<T: Default> {
    _marker: PhantomData<T>,
}

#[derive(Component, Default, Clone)]
struct PlaySoundButton<T: Default> {
    _marker: PhantomData<T>,
}

#[derive(Component, Default, Clone)]
struct StartLoopButton<T: Default> {
    _marker: PhantomData<T>,
}

#[derive(Component, Clone)]
struct ChangeVolumeButton<T> {
    louder: bool,
    _marker: PhantomData<T>,
}

#[derive(Component, Default, Clone)]
struct StopButton<T: Default> {
    _marker: PhantomData<T>,
}

#[derive(Resource, Component, Default, Clone)]
struct FirstChannel;
#[derive(Resource, Component, Default, Clone)]
struct SecondChannel;
#[derive(Resource, Component, Default, Clone)]
struct ThirdChannel;

#[derive(Resource)]
struct AudioHandles {
    loop_handle: Handle<AudioSource>,
    sound_handle: Handle<AudioSource>,
}

#[derive(Resource)]
struct ChannelAudioState<T> {
    stopped: bool,
    paused: bool,
    loop_started: bool,
    volume: f64,
    _marker: PhantomData<T>,
}

impl<T> Default for ChannelAudioState<T> {
    fn default() -> Self {
        ChannelAudioState {
            volume: 1.0,
            stopped: true,
            loop_started: false,
            paused: false,
            _marker: PhantomData::<T>,
        }
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const DISABLED_BUTTON: Color = Color::rgb(0.5, 0.5, 0.5);

fn prepare_audio_and_ui(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let loop_handle = asset_server.load("sounds/loop.ogg");
    let sound_handle = asset_server.load("sounds/sound.ogg");

    set_up_ui(&mut commands, asset_server);

    commands.insert_resource(AudioHandles {
        loop_handle,
        sound_handle,
    });
    commands.insert_resource(ChannelAudioState::<FirstChannel>::default());
    commands.insert_resource(ChannelAudioState::<SecondChannel>::default());
    commands.insert_resource(ChannelAudioState::<ThirdChannel>::default());
}

fn set_up_ui(commands: &mut Commands, asset_server: ResMut<AssetServer>) {
    let font = asset_server.load("fonts/monogram.ttf");
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            build_button_row::<FirstChannel>(parent, &font, 1);
            build_button_row::<SecondChannel>(parent, &font, 2);
            build_button_row::<ThirdChannel>(parent, &font, 3);
        });
}

fn build_button_row<T: Component + Default + Clone>(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    channel_index: u8,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.),
                height: Val::Percent(33.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(120.0),
                        height: Val::Percent(100.),
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
                                value: format!("Channel {}", 4 - channel_index),
                                style: TextStyle {
                                    font_size: 20.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                    font: font.clone(),
                                },
                            }],
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                });
            spawn_button(
                parent,
                "Sound",
                DISABLED_BUTTON.into(),
                PlaySoundButton::<T>::default(),
                font.clone(),
            );
            spawn_button(
                parent,
                "Loop",
                DISABLED_BUTTON.into(),
                StartLoopButton::<T>::default(),
                font.clone(),
            );
            spawn_button(
                parent,
                "Pause",
                DISABLED_BUTTON.into(),
                PlayPauseButton::<T>::default(),
                font.clone(),
            );
            spawn_button(
                parent,
                "Vol. up",
                NORMAL_BUTTON.into(),
                ChangeVolumeButton::<T> {
                    louder: true,
                    _marker: PhantomData,
                },
                font.clone(),
            );
            spawn_button(
                parent,
                "Vol. down",
                NORMAL_BUTTON.into(),
                ChangeVolumeButton::<T> {
                    louder: false,
                    _marker: PhantomData,
                },
                font.clone(),
            );
            spawn_button(
                parent,
                "Stop",
                DISABLED_BUTTON.into(),
                StopButton::<T>::default(),
                font.clone(),
            );
        });
}

fn spawn_button<T: Component + Clone>(
    parent: &mut ChildBuilder,
    text: &str,
    background_color: BackgroundColor,
    marker: T,
    font: Handle<Font>,
) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(65.0),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color,
            ..Default::default()
        })
        .insert(marker.clone())
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: text.to_string(),
                            style: TextStyle {
                                font_size: 20.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                font: font.clone(),
                            },
                        }],
                        justify: JustifyText::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(marker);
        });
}
