use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};

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
        .add_resource(AudioState { paused: false })
        .init_resource::<ButtonMaterials>()
        .add_startup_system(set_up_ui.system())
        .add_startup_system(start_audio.system())
        .add_system(play_pause_button.system());

    app.run();
}

struct PlayPauseButton;

struct PlayPauseButtonText;

struct AudioState {
    paused: bool,
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
}

impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
        }
    }
}

fn set_up_ui(
    commands: &mut Commands,
    asset_server: ResMut<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    let font = asset_server.load("fonts/monogram.ttf");
    commands
        .spawn(CameraUiBundle::default())
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        margin: Rect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: button_materials.normal.clone(),
                    ..Default::default()
                })
                .with(PlayPauseButton)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle {
                            text: Text {
                                value: "Pause".to_string(),
                                font,
                                style: TextStyle {
                                    font_size: 20.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        })
                        .with(PlayPauseButtonText);
                });
        });
}

fn start_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let music: Handle<AudioSource> = asset_server.load("sounds/background.ogg");
    audio.play_looped(music);
}

fn play_pause_button(
    button_materials: Res<ButtonMaterials>,
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>),
        (Mutated<Interaction>, With<PlayPauseButton>),
    >,
    mut play_pause_text: Query<&mut Text, With<PlayPauseButtonText>>,
) {
    for (interaction, mut material) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                if audio_state.paused {
                    audio.resume();
                    for mut text in play_pause_text.iter_mut() {
                        text.value = "Pause".to_owned();
                    }
                } else {
                    audio.pause();
                    for mut text in play_pause_text.iter_mut() {
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
