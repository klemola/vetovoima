use bevy::{app::AppExit, prelude::*};

use crate::app::{cursor_visible, AppState, ButtonPress, VetovoimaColor, APP_NAME};

const BUTTON_COLOR: Color = VetovoimaColor::BLUEISH_DARK;
const BUTTON_COLOR_HOVER: Color = VetovoimaColor::BLUEISH_MID;
const BUTTON_ACTIVE_COLOR: Color = VetovoimaColor::BLUEISH_LIGHT;
static NEW_GAME_BUTTON_LABEL: &str = "New game";
static EXIT_BUTTON_LABEL: &str = "Exit";

#[derive(Component)]
pub enum MenuEvent {
    EnterMenu,
    BeginNewGame,
}

#[derive(Component, PartialEq, Clone, Copy, Debug)]
enum MenuButton {
    NewGame,
    Exit,
}

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
struct SelectedButton(Option<MenuButton>);

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MenuEvent>()
            .insert_resource(SelectedButton(None))
            .add_system_set(
                SystemSet::on_enter(AppState::InMenu)
                    .with_system(show_menu)
                    .with_system(cursor_visible::<true>),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InMenu)
                    .with_system(mouse_interaction)
                    .with_system(selected_button_change)
                    .with_system(button_press),
            )
            .add_system_set(SystemSet::on_exit(AppState::InMenu).with_system(hide_menu));
    }
}

fn show_menu(
    mut commands: Commands,
    mut menu_event: EventWriter<MenuEvent>,
    mut selected_button: ResMut<SelectedButton>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("VT323-Regular.ttf");
    let font_size = 64.0;

    selected_button.0 = None;
    menu_event.send(MenuEvent::EnterMenu);

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
                                font_size,
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
                                font,
                                font_size,
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

fn mouse_interaction(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut UiColor),
        (Changed<Interaction>, With<MenuButton>),
    >,
    selected_button: Res<SelectedButton>,
    mut app_state: ResMut<State<AppState>>,
    mut menu_event: EventWriter<MenuEvent>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, button, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = BUTTON_ACTIVE_COLOR.into();

                match button {
                    MenuButton::NewGame => {
                        menu_event.send(MenuEvent::BeginNewGame);

                        app_state
                            .set(AppState::InitGame)
                            .expect("Could not start the game")
                    }
                    MenuButton::Exit => exit.send(AppExit),
                };
            }
            Interaction::Hovered => {
                *color = BUTTON_COLOR_HOVER.into();
            }
            Interaction::None => {
                *color = match selected_button.0 {
                    Some(selected) if selected == *button => BUTTON_ACTIVE_COLOR.into(),

                    _ => BUTTON_COLOR.into(),
                };
            }
        }
    }
}

fn selected_button_change(
    mut menu_buttons_query: Query<(&MenuButton, &mut UiColor)>,
    selected_button: Res<SelectedButton>,
) {
    if selected_button.is_changed() {
        for (button, mut color) in menu_buttons_query.iter_mut() {
            *color = match selected_button.0 {
                Some(selected) if selected == *button => BUTTON_ACTIVE_COLOR.into(),

                _ => BUTTON_COLOR.into(),
            };
        }
    }
}

fn button_press(
    button_press: Res<ButtonPress>,
    mut selected_button: ResMut<SelectedButton>,
    mut app_state: ResMut<State<AppState>>,
    mut menu_event: EventWriter<MenuEvent>,
    mut exit: EventWriter<AppExit>,
) {
    if !button_press.is_changed() {
        return;
    }

    if button_press.main_control_pressed {
        match selected_button.0 {
            Some(MenuButton::NewGame) => {
                menu_event.send(MenuEvent::BeginNewGame);

                app_state
                    .set(AppState::InitGame)
                    .expect("Could not start the game")
            }

            Some(MenuButton::Exit) => exit.send(AppExit),

            _ => (),
        }
    } else if button_press.up_pressed || button_press.down_pressed {
        selected_button.0 = Some(select_next_button(selected_button.0));
    }
}

fn select_next_button(selected_button: Option<MenuButton>) -> MenuButton {
    match selected_button {
        Some(button) => {
            if button == MenuButton::NewGame {
                MenuButton::Exit
            } else {
                MenuButton::NewGame
            }
        }

        None => MenuButton::NewGame,
    }
}
