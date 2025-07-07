use bevy::{app::AppExit, prelude::*};

use crate::{
    app::{cursor_visible, AppState, ButtonPress, UiConfig, VetovoimaColor, APP_NAME},
    game::GameLevel,
};

const BUTTON_COLOR: Color = VetovoimaColor::BLUEISH_DARK;
const BUTTON_COLOR_HOVER: Color = VetovoimaColor::BLUEISH_MID;
const BUTTON_ACTIVE_COLOR: Color = VetovoimaColor::BLUEISH_LIGHT;
static NEW_GAME_BUTTON_LABEL: &str = "New game";
static EXIT_BUTTON_LABEL: &str = "Exit";

#[derive(Event)]
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
            .add_systems(
                OnEnter(AppState::InMenu),
                (show_menu, cursor_visible::<true>),
            )
            .add_systems(
                Update,
                (
                    mouse_interaction,
                    selected_button_change,
                    button_press,
                    init_game,
                ),
            )
            .add_systems(OnExit(AppState::InMenu), hide_menu);
    }
}

fn show_menu(
    mut commands: Commands,
    mut menu_event: EventWriter<MenuEvent>,
    mut selected_button: ResMut<SelectedButton>,
    asset_server: Res<AssetServer>,
    ui_config: Res<UiConfig>,
) {
    let font = asset_server.load(ui_config.font_filename);
    let button_width = 400.0 * ui_config.scale_multiplier;
    let button_height = 80.0 * ui_config.scale_multiplier;
    let margin = 10.0 * ui_config.scale_multiplier;

    selected_button.0 = None;
    menu_event.send(MenuEvent::EnterMenu);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..Default::default()
            },
            BackgroundColor(VetovoimaColor::BLACKISH.into()),
        ))
        .insert(MainMenu)
        .with_children(|menu_node| {
            menu_node
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(120.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(margin * 2.0)),
                        ..default()
                    },
                    BackgroundColor(VetovoimaColor::BLACKISH.into()),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(APP_NAME),
                        TextFont {
                            font: font.clone(),
                            font_size: ui_config.font_size_app_title,
                            ..Default::default()
                        },
                        TextColor(VetovoimaColor::WHITEISH),
                    ));
                });

            menu_node
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(button_width),
                        height: Val::Px(button_height),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(margin)),
                        ..default()
                    },
                    BackgroundColor(BUTTON_COLOR),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(NEW_GAME_BUTTON_LABEL),
                        TextFont {
                            font: font.clone(),
                            font_size: ui_config.font_size_menu_item,
                            ..Default::default()
                        },
                        TextColor(VetovoimaColor::WHITEISH),
                    ));
                })
                .insert(MenuButton::NewGame);

            menu_node
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(button_width),
                        height: Val::Px(button_height),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(margin)),
                        ..default()
                    },
                    BackgroundColor(BUTTON_COLOR),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(EXIT_BUTTON_LABEL),
                        TextFont {
                            font: font.clone(),
                            font_size: ui_config.font_size_menu_item,
                            ..Default::default()
                        },
                        TextColor(VetovoimaColor::WHITEISH),
                    ));
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
            Interaction::Pressed => {
                *color = BUTTON_ACTIVE_COLOR.into();

                match button {
                    MenuButton::NewGame => {
                        menu_event.send(MenuEvent::BeginNewGame);
                    }
                    MenuButton::Exit => {
                        exit.send(AppExit::Success);
                    }
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
            Some(MenuButton::NewGame) => {
                menu_event.send(MenuEvent::BeginNewGame);
            }
            Some(MenuButton::Exit) => {
                exit.send(AppExit::Success);
            }

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
    for event in menu_event.read() {
        match event {
            MenuEvent::BeginNewGame => {
                commands.remove_resource::<GameLevel>();
                app_state.set(AppState::LoadingLevel);
            }

            _ => (),
        }
    }
}
