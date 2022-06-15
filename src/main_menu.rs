use bevy::{app::AppExit, prelude::*};

use crate::app::{cursor_visible, AppState, VetovoimaColor, APP_NAME};

const BUTTON_COLOR: Color = VetovoimaColor::BLUEISH_DARK;
const BUTTON_COLOR_HOVER: Color = VetovoimaColor::BLUEISH_MID;
const BUTTON_ACTIVE_COLOR: Color = VetovoimaColor::BLUEISH_LIGHT;
static NEW_GAME_BUTTON_LABEL: &str = "New game";
static EXIT_BUTTON_LABEL: &str = "Exit";

#[derive(Component)]
enum MenuButton {
    NewGame,
    Exit,
    SimulationMode,
}

#[derive(Component)]
struct MainMenu;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::InMenu)
                .with_system(show_menu)
                .with_system(cursor_visible::<true>),
        )
        .add_system_set(
            SystemSet::on_update(AppState::InMenu)
                .with_system(menu_button_state)
                .with_system(menu_button_event),
        )
        .add_system_set(SystemSet::on_exit(AppState::InMenu).with_system(hide_menu));
    }
}

fn show_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("VT323-Regular.ttf");
    let font_size = 64.0;

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                padding: Rect::all(Val::Px(10.0)),
                ..Default::default()
            },
            color: VetovoimaColor::BLACKISH.into(),
            ..Default::default()
        })
        .insert(MainMenu)
        .with_children(|menu_node| {
            menu_node
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(600.0), Val::Px(120.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(20.0)),
                        ..default()
                    },
                    color: VetovoimaColor::BLACKISH.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            APP_NAME,
                            TextStyle {
                                font: font.clone(),
                                font_size: font_size * 1.6,
                                color: VetovoimaColor::WHITEISH,
                            },
                            Default::default(),
                        ),
                        ..default()
                    });
                });

            menu_node
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(400.0), Val::Px(80.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(10.0)),
                        ..default()
                    },
                    color: BUTTON_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            NEW_GAME_BUTTON_LABEL,
                            TextStyle {
                                font: font.clone(),
                                font_size: font_size,
                                color: VetovoimaColor::WHITEISH,
                            },
                            Default::default(),
                        ),
                        ..default()
                    });
                })
                .insert(MenuButton::NewGame);

            menu_node
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(400.0), Val::Px(80.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(10.0)),
                        ..default()
                    },
                    color: BUTTON_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            EXIT_BUTTON_LABEL,
                            TextStyle {
                                font: font.clone(),
                                font_size: font_size,
                                color: VetovoimaColor::WHITEISH,
                            },
                            Default::default(),
                        ),
                        ..default()
                    });
                })
                .insert(MenuButton::Exit);
        });
}

fn hide_menu(mut commands: Commands, menu: Query<Entity, With<MainMenu>>) {
    let menu = menu.get_single().expect("Could not hide the menu");
    commands.entity(menu).despawn_recursive();
}

fn menu_button_state(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<MenuButton>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = BUTTON_ACTIVE_COLOR.into();
            }
            Interaction::Hovered => {
                *color = BUTTON_COLOR_HOVER.into();
            }
            Interaction::None => {
                *color = BUTTON_COLOR.into();
            }
        }
    }
}

fn menu_button_event(
    interaction_query: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut app_state: ResMut<State<AppState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, button) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => match button {
                MenuButton::NewGame => app_state
                    .set(AppState::InitGame)
                    .expect("Could not start the game"),
                MenuButton::Exit => exit.send(AppExit),
                MenuButton::SimulationMode => {
                    app_state
                        .set(AppState::ObserveSimulation)
                        .expect("Could not start the observe simulation mode");
                }
            },

            _ => (),
        }
    }
}
