use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::schedule::ScheduleConfigs;
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

fn create_row_systems<C: Component + Default>(
) -> ScheduleConfigs<Box<(dyn bevy::prelude::System<In = (), Out = Result<(), BevyError>> + 'static)>>
{
    (
        stop_button::<C>,
        loop_button::<C>,
        volume_buttons::<C>,
        play_sound_button::<C>,
        play_pause_button::<C>,
    )
        .into_configs()
}

fn play_pause_button<T: Component + Default>(
    channel: Res<AudioChannel<T>>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
    time: Res<Time>,
    mut last_action: ResMut<LastAction>,
    mut interaction_query: Query<(&Interaction, &mut ImageNode), With<PlayPauseButton<T>>>,
    mut play_pause_text: Query<&mut TextSpan, With<PlayPauseButton<T>>>,
) -> Result {
    let (interaction, mut image) = interaction_query.single_mut()?;
    image.color = if channel_state.stopped {
        DISABLED_BUTTON
    } else if interaction == &Interaction::Hovered {
        HOVERED_BUTTON
    } else {
        NORMAL_BUTTON
    };
    let mut text = play_pause_text.single_mut()?;
    text.0 = if channel_state.paused {
        "Play".to_owned()
    } else {
        "Pause".to_owned()
    };
    if channel_state.stopped {
        return Ok(());
    }
    if interaction == &Interaction::Pressed && last_action.action(&time) {
        if channel_state.paused {
            channel.resume();
        } else {
            channel.pause();
        }
        channel_state.paused = !channel_state.paused;
    }

    Ok(())
}

fn stop_button<T: Component + Default>(
    channel: Res<AudioChannel<T>>,
    time: Res<Time>,
    mut last_action: ResMut<LastAction>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
    mut interaction_query: Query<(&Interaction, &mut ImageNode), With<StopButton<T>>>,
) -> Result {
    let (interaction, mut image) = interaction_query.single_mut()?;
    image.color = if channel_state.stopped {
        DISABLED_BUTTON
    } else if interaction == &Interaction::Hovered {
        HOVERED_BUTTON
    } else {
        NORMAL_BUTTON
    };
    if channel_state.stopped {
        return Ok(());
    }
    if interaction == &Interaction::Pressed && last_action.action(&time) {
        channel.stop();
        *channel_state = ChannelAudioState::<T>::default();
    }

    Ok(())
}

fn loop_button<T: Component + Default>(
    channel: Res<AudioChannel<T>>,
    time: Res<Time>,
    mut last_action: ResMut<LastAction>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
    audio_handles: Res<AudioHandles>,
    mut interaction_query: Query<(&Interaction, &mut ImageNode), With<StartLoopButton<T>>>,
) -> Result {
    let (interaction, mut image) = interaction_query.single_mut()?;
    image.color = if !channel_state.loop_started {
        if interaction == &Interaction::Hovered {
            HOVERED_BUTTON
        } else {
            NORMAL_BUTTON
        }
    } else {
        DISABLED_BUTTON
    };
    if channel_state.loop_started {
        return Ok(());
    }
    if interaction == &Interaction::Pressed && last_action.action(&time) {
        channel_state.loop_started = true;
        channel_state.stopped = false;
        channel.play(audio_handles.loop_handle.clone()).looped();
    }

    Ok(())
}

fn play_sound_button<T: Component + Default>(
    channel: Res<AudioChannel<T>>,
    time: Res<Time>,
    mut last_action: ResMut<LastAction>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
    audio_handles: Res<AudioHandles>,
    mut interaction_query: Query<(&Interaction, &mut ImageNode), With<PlaySoundButton<T>>>,
) -> Result {
    let (interaction, mut image) = interaction_query.single_mut()?;
    image.color = if interaction == &Interaction::Hovered {
        HOVERED_BUTTON
    } else {
        NORMAL_BUTTON
    };
    if interaction == &Interaction::Pressed && last_action.action(&time) {
        channel_state.paused = false;
        channel_state.stopped = false;
        channel.play(audio_handles.sound_handle.clone());
    }

    Ok(())
}

fn volume_buttons<T: Component + Default>(
    channel: Res<AudioChannel<T>>,
    time: Res<Time>,
    mut last_action: ResMut<LastAction>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
    mut interaction_query: Query<(&Interaction, &mut ImageNode, &ChangeVolumeButton<T>)>,
) {
    for (interaction, mut image, volume) in &mut interaction_query {
        image.color = if interaction == &Interaction::Hovered {
            HOVERED_BUTTON
        } else {
            NORMAL_BUTTON
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
            channel.set_volume(Decibels(channel_state.volume as f32));
        }
    }
}

#[derive(Resource, Default)]
struct LastAction(f64);

impl LastAction {
    fn action(&mut self, time: &Time) -> bool {
        if time.elapsed_secs_f64() - self.0 < 0.2 {
            return false;
        }
        self.0 = time.elapsed_secs_f64();

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

const NORMAL_BUTTON: Color = Color::linear_rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::linear_rgb(0.25, 0.25, 0.25);
const DISABLED_BUTTON: Color = Color::linear_rgb(0.5, 0.5, 0.5);

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
    commands.spawn(Camera2d);
    commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..Default::default()
        })
        .with_children(|parent| {
            build_button_row::<FirstChannel>(parent, &font, 1);
            build_button_row::<SecondChannel>(parent, &font, 2);
            build_button_row::<ThirdChannel>(parent, &font, 3);
        });
}

fn build_button_row<T: Component + Default + Clone>(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font: &Handle<Font>,
    channel_index: u8,
) {
    parent
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.),
            height: Val::Percent(33.3),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Px(120.0),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(format!("Channel {}", 4 - channel_index)),
                        TextFont {
                            font: font.clone(),
                            font_size: 20.0,
                            ..Default::default()
                        },
                        TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    ));
                });
            spawn_button(
                parent,
                "Sound",
                DISABLED_BUTTON,
                PlaySoundButton::<T>::default(),
                font.clone(),
            );
            spawn_button(
                parent,
                "Loop",
                DISABLED_BUTTON,
                StartLoopButton::<T>::default(),
                font.clone(),
            );
            spawn_button(
                parent,
                "Pause",
                DISABLED_BUTTON,
                PlayPauseButton::<T>::default(),
                font.clone(),
            );
            spawn_button(
                parent,
                "Vol. up",
                NORMAL_BUTTON,
                ChangeVolumeButton::<T> {
                    louder: true,
                    _marker: PhantomData,
                },
                font.clone(),
            );
            spawn_button(
                parent,
                "Vol. down",
                NORMAL_BUTTON,
                ChangeVolumeButton::<T> {
                    louder: false,
                    _marker: PhantomData,
                },
                font.clone(),
            );
            spawn_button(
                parent,
                "Stop",
                DISABLED_BUTTON,
                StopButton::<T>::default(),
                font.clone(),
            );
        });
}

fn spawn_button<T: Component + Clone>(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    text: &str,
    color: Color,
    marker: T,
    font: Handle<Font>,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(100.0),
                height: Val::Px(65.0),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ImageNode::default().with_color(color),
            Button,
        ))
        .insert(marker.clone())
        .with_children(|parent| {
            parent
                .spawn((
                    Text::new(String::new()),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
                        ..Default::default()
                    },
                    TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    TextLayout::new_with_justify(JustifyText::Center),
                ))
                .with_child((TextSpan::new(text), marker));
        });
}
