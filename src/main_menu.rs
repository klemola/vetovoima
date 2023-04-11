use bevy::{app::AppExit, prelude::*};

use crate::{
    app::{cursor_visible, AppState, ButtonPress, VetovoimaColor, APP_NAME},
    game::GameLevel,
};

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

#[derive(Component, Resource)]
struct SelectedButton(Option<MenuButton>);

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MenuEvent>()
            .insert_resource(SelectedButton(None))
            .add_systems((show_menu, cursor_visible::<true>).in_schedule(OnEnter(AppState::InMenu)))
            .add_systems(
                (
                    mouse_interaction,
                    selected_button_change,
                    button_press,
                    init_game,
                )
                    .in_set(OnUpdate(AppState::InMenu)),
            )
            .add_system(hide_menu.in_schedule(OnExit(AppState::InMenu)));
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
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..Default::default()
            },
            background_color: VetovoimaColor::BLACKISH.into(),
            ..Default::default()
        })
        .insert(MainMenu)
        .with_children(|menu_node| {
            menu_node
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(600.0), Val::Px(120.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    background_color: VetovoimaColor::BLACKISH.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            APP_NAME,
                            TextStyle {
                                font: font.clone(),
                                font_size: font_size * 1.6,
                                color: VetovoimaColor::WHITEISH,
                            },
                        ),
                        ..default()
                    });
                });

            menu_node
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(400.0), Val::Px(80.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: BUTTON_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            NEW_GAME_BUTTON_LABEL,
                            TextStyle {
                                font: font.clone(),
                                font_size,
                                color: VetovoimaColor::WHITEISH,
                            },
                        ),
                        ..default()
                    });
                })
                .insert(MenuButton::NewGame);

            menu_node
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(400.0), Val::Px(80.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: BUTTON_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            EXIT_BUTTON_LABEL,
                            TextStyle {
                                font,
                                font_size,
                                color: VetovoimaColor::WHITEISH,
                            },
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
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuButton>),
    >,
    selected_button: Res<SelectedButton>,
    mut menu_event: EventWriter<MenuEvent>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, button, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = BUTTON_ACTIVE_COLOR.into();

                match button {
                    MenuButton::NewGame => menu_event.send(MenuEvent::BeginNewGame),
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
    mut menu_buttons_query: Query<(&MenuButton, &mut BackgroundColor)>,
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
    mut menu_event: EventWriter<MenuEvent>,
    mut exit: EventWriter<AppExit>,
) {
    if !button_press.is_changed() {
        return;
    }

    if button_press.main_control_pressed {
        match selected_button.0 {
            Some(MenuButton::NewGame) => menu_event.send(MenuEvent::BeginNewGame),
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

fn init_game(
    mut commands: Commands,
    mut menu_event: EventReader<MenuEvent>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    // Effectively resets the game (start from level 1)

    for event in menu_event.iter() {
        match event {
            MenuEvent::BeginNewGame => {
                commands.remove_resource::<GameLevel>();
                app_state.set(AppState::LoadingLevel);
            }

            _ => (),
        }
    }
}
