use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};
use bevy_kira_audio::AudioApp;
use bevy_kira_audio::prelude::*;
use std::clone::Clone;
use std::marker::PhantomData;

// This is a bigger example with a GUI for full control over three audio channels
fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, prepare_audio_and_ui)
        .add_audio_channel::<FirstChannel>()
        .add_audio_channel::<SecondChannel>()
        .add_audio_channel::<ThirdChannel>();
    add_channel_row::<FirstChannel>(&mut app);
    add_channel_row::<SecondChannel>(&mut app);
    add_channel_row::<ThirdChannel>(&mut app);

    app.run();
}

fn add_channel_row<C: Component + Default>(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_play_pause_button::<C>,
            update_stop_button::<C>,
            update_loop_button::<C>,
            update_play_sound_button::<C>,
            update_volume_buttons::<C>,
        ),
    )
    .add_observer(play_pause::<C>)
    .add_observer(stop::<C>)
    .add_observer(start_loop::<C>)
    .add_observer(play_sound::<C>)
    .add_observer(change_volume::<C>);
}

fn update_play_pause_button<T: Component + Default>(
    channel_state: Res<ChannelAudioState<T>>,
    mut button: Query<(&Hovered, &mut BackgroundColor), With<PlayPauseButton<T>>>,
    mut play_pause_text: Query<&mut TextSpan, With<PlayPauseButton<T>>>,
) -> Result {
    let (hovered, mut background_color) = button.single_mut()?;
    background_color.0 = if channel_state.stopped {
        DISABLED_BUTTON
    } else if hovered.get() {
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

    Ok(())
}

fn play_pause<T: Component + Default>(
    activate: On<Activate>,
    buttons: Query<(), With<PlayPauseButton<T>>>,
    channel: Res<AudioChannel<T>>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
) {
    if !buttons.contains(activate.entity) || channel_state.stopped {
        return;
    }
    if channel_state.paused {
        channel.resume();
    } else {
        channel.pause();
    }
    channel_state.paused = !channel_state.paused;
}

fn update_stop_button<T: Component + Default>(
    channel_state: Res<ChannelAudioState<T>>,
    mut button: Query<(&Hovered, &mut BackgroundColor), With<StopButton<T>>>,
) -> Result {
    let (hovered, mut background_color) = button.single_mut()?;
    background_color.0 = if channel_state.stopped {
        DISABLED_BUTTON
    } else if hovered.get() {
        HOVERED_BUTTON
    } else {
        NORMAL_BUTTON
    };

    Ok(())
}

fn stop<T: Component + Default>(
    activate: On<Activate>,
    buttons: Query<(), With<StopButton<T>>>,
    channel: Res<AudioChannel<T>>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
) {
    if !buttons.contains(activate.entity) || channel_state.stopped {
        return;
    }
    channel.stop();
    *channel_state = ChannelAudioState::<T>::default();
}

fn update_loop_button<T: Component + Default>(
    channel_state: Res<ChannelAudioState<T>>,
    mut button: Query<(&Hovered, &mut BackgroundColor), With<StartLoopButton<T>>>,
) -> Result {
    let (hovered, mut background_color) = button.single_mut()?;
    background_color.0 = if channel_state.loop_started {
        DISABLED_BUTTON
    } else if hovered.get() {
        HOVERED_BUTTON
    } else {
        NORMAL_BUTTON
    };

    Ok(())
}

fn start_loop<T: Component + Default>(
    activate: On<Activate>,
    buttons: Query<(), With<StartLoopButton<T>>>,
    channel: Res<AudioChannel<T>>,
    audio_handles: Res<AudioHandles>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
) {
    if !buttons.contains(activate.entity) || channel_state.loop_started {
        return;
    }
    channel_state.loop_started = true;
    channel_state.stopped = false;
    channel.play(audio_handles.loop_handle.clone()).looped();
}

fn update_play_sound_button<T: Component + Default>(
    mut button: Query<(&Hovered, &mut BackgroundColor), With<PlaySoundButton<T>>>,
) -> Result {
    let (hovered, mut background_color) = button.single_mut()?;
    background_color.0 = if hovered.get() {
        HOVERED_BUTTON
    } else {
        NORMAL_BUTTON
    };

    Ok(())
}

fn play_sound<T: Component + Default>(
    activate: On<Activate>,
    buttons: Query<(), With<PlaySoundButton<T>>>,
    channel: Res<AudioChannel<T>>,
    audio_handles: Res<AudioHandles>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
) {
    if !buttons.contains(activate.entity) {
        return;
    }
    channel_state.paused = false;
    channel_state.stopped = false;
    channel
        .play(audio_handles.sound_handle.clone())
        .with_volume(channel_state.volume);
}

fn update_volume_buttons<T: Component + Default>(
    mut buttons: Query<(&Hovered, &mut BackgroundColor), With<ChangeVolumeButton<T>>>,
) {
    for (hovered, mut background_color) in &mut buttons {
        background_color.0 = if hovered.get() {
            HOVERED_BUTTON
        } else {
            NORMAL_BUTTON
        };
    }
}

fn change_volume<T: Component + Default>(
    activate: On<Activate>,
    buttons: Query<&ChangeVolumeButton<T>>,
    channel: Res<AudioChannel<T>>,
    mut channel_state: ResMut<ChannelAudioState<T>>,
) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };
    if button.louder {
        channel_state.volume += 2.;
    } else {
        channel_state.volume = (channel_state.volume - 2.).max(-60.);
    }
    println!("{}", channel_state.volume);
    channel.set_volume(channel_state.volume);
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

#[derive(Resource, Default, Clone)]
struct FirstChannel;
#[derive(Resource, Default, Clone)]
struct SecondChannel;
#[derive(Resource, Default, Clone)]
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
    volume: f32,
    _marker: PhantomData<T>,
}

impl<T> Default for ChannelAudioState<T> {
    fn default() -> Self {
        ChannelAudioState {
            volume: 0.0,
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
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..Default::default()
            },
            BackgroundColor(Color::BLACK),
        ))
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
                            font: font.into(),
                            font_size: 20.0.into(),
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
            BackgroundColor(color),
            Button,
            Hovered::default(),
        ))
        .insert(marker.clone())
        .with_children(|parent| {
            parent
                .spawn((
                    Text::new(String::new()),
                    TextFont {
                        font: font.into(),
                        font_size: 20.0.into(),
                        ..Default::default()
                    },
                    TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    TextLayout::justify(Justify::Center),
                ))
                .with_child((TextSpan::new(text), marker));
        });
}
