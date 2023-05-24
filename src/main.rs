mod app;
mod devtools;
mod game;
mod game_over;
mod main_menu;
mod simulation;
mod sounds;
// Temporary local version of the "bevy-rust-arcade" crate to avoid Bevy version mismatch
mod arcade_cabinet;

use bevy::{
    ecs::event::Events,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::camera::ScalingMode,
    window::{PrimaryWindow, WindowMode, WindowResized},
};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};

use app::{AppState, ButtonPress, UiConfig, VetovoimaColor, APP_NAME, PIXELS_PER_METER};
use arcade_cabinet::{ArcadeInput, ArcadeInputEvent, RustArcadePlugin};
use devtools::DevTools;
use game::GamePlugin;
use game_over::GameOverPlugin;
use main_menu::MainMenuPlugin;
use simulation::SimulationPlugin;
use sounds::SoundsPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(VetovoimaColor::BLACKISH))
        .insert_resource(ButtonPress::default())
        .insert_resource(UiConfig::default())
        .add_state::<AppState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: APP_NAME.into(),
                mode: WindowMode::Fullscreen,
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .add_plugin(RustArcadePlugin)
        .add_plugin(SoundsPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(SimulationPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(GameOverPlugin)
        .add_plugins(DevTools)
        .add_startup_system(app_setup)
        .add_system(app_controls)
        .add_system(keyboard_input)
        .add_system(arcade_input)
        .add_system(window_resize)
        .run();
}

fn app_setup(mut commands: Commands, primary_window: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };
    let (projection_scale, _window_height) = window_to_projection_scale(window, None);

    let mut game_camera = Camera2dBundle::default();

    game_camera.projection.scaling_mode = ScalingMode::FixedVertical(2.0);
    game_camera.projection.scale = projection_scale;

    commands.spawn(game_camera);
}

fn app_controls(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    button_press: Res<ButtonPress>,
    app_state: ResMut<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let should_go_to_menu =
        keyboard_input.just_released(KeyCode::Escape) || button_press.start_pressed;

    if should_go_to_menu && app_state.0 != AppState::InMenu {
        next_app_state.set(AppState::InMenu);
        keyboard_input.reset(KeyCode::Escape);
    }
}

fn arcade_input(
    mut arcade_input_events: EventReader<ArcadeInputEvent>,
    mut button_press: ResMut<ButtonPress>,
) {
    for event in arcade_input_events.iter() {
        let is_pressed = event.value == 1.0;

        match event.arcade_input {
            ArcadeInput::ButtonFront1 => button_press.select_pressed = is_pressed,

            ArcadeInput::ButtonFront2 => button_press.start_pressed = is_pressed,

            ArcadeInput::JoyButton => button_press.main_control_pressed = is_pressed,

            ArcadeInput::JoyUp => button_press.up_pressed = is_pressed,

            ArcadeInput::JoyDown => button_press.down_pressed = is_pressed,

            ArcadeInput::JoyLeft => button_press.left_pressed = is_pressed,

            ArcadeInput::JoyRight => button_press.right_pressed = is_pressed,

            _ => (),
        }
    }
}

fn keyboard_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut button_press: ResMut<ButtonPress>,
) {
    for event in keyboard_events.iter() {
        let is_pressed = ButtonState::is_pressed(&event.state);

        match event.key_code {
            Some(KeyCode::Up) => button_press.up_pressed = is_pressed,
            Some(KeyCode::Down) => button_press.down_pressed = is_pressed,
            Some(KeyCode::Left) => button_press.left_pressed = is_pressed,
            Some(KeyCode::Right) => button_press.right_pressed = is_pressed,
            Some(KeyCode::Return) => button_press.main_control_pressed = is_pressed,
            Some(KeyCode::Escape) => button_press.select_pressed = is_pressed,

            _ => (),
        }
    }
}

fn window_resize(
    resize_event: Res<Events<WindowResized>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut query: Query<&mut OrthographicProjection, With<Camera2d>>,
    mut ui_config: ResMut<UiConfig>,
) {
    let mut reader = resize_event.get_reader();
    for event in reader.iter(&resize_event) {
        for mut projection in query.iter_mut() {
            let Ok(window) = primary_window.get_single() else {
                return;
            };
            let (projection_scale, window_height) =
                window_to_projection_scale(window, Some(event.height));
            // The world created at 4k, then scaled to fit the practical resolution
            let scale_ratio = 2160.0 / window_height;
            let scaled_ui_config = UiConfig::scale(scale_ratio);

            projection.scale = projection_scale;
            // the multiplier leaves some margin around the visuals
            projection.scaling_mode = ScalingMode::FixedVertical(scale_ratio * 1.1);
            *ui_config = scaled_ui_config;

            if app_state.0 == AppState::Init {
                next_app_state.set(AppState::InMenu);
            }
        }
    }
}

fn window_to_projection_scale(window: &Window, height_override: Option<f32>) -> (f32, f32) {
    let height = if window.mode == WindowMode::Windowed {
        height_override.unwrap_or_else(|| window.height())
    } else {
        window.height()
    };

    (height / 2.0, height)
}
