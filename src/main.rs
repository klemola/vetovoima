mod app;
mod devtools;
mod game;
mod game_over;
mod main_menu;
mod simulation;

use bevy::{
    ecs::event::Events,
    prelude::*,
    render::camera::{Camera2d, ScalingMode},
    window::{WindowMode, WindowResized},
};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rust_arcade::{ArcadeInput, ArcadeInputEvent, RustArcadePlugin};

use app::{AppState, ButtonPress, VetovoimaColor, APP_NAME, PIXELS_PER_METER};
use devtools::DevTools;
use game::GamePlugin;
use game_over::GameOverPlugin;
use main_menu::MainMenuPlugin;
use simulation::SimulationPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(VetovoimaColor::BLACKISH))
        .insert_resource(WindowDescriptor {
            title: APP_NAME.into(),
            mode: WindowMode::Fullscreen,
            ..default()
        })
        .insert_resource(ButtonPress::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .add_plugin(RustArcadePlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(SimulationPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(GameOverPlugin)
        .add_plugins(DevTools)
        .add_state(AppState::InMenu)
        .add_startup_system(app_setup)
        .add_system(app_controls)
        .add_system(update_button_input)
        .add_system(window_resize)
        .run();
}

fn app_setup(mut commands: Commands, window: Res<Windows>) {
    let window = window.primary();
    let projection_scale = window_to_projection_scale(window, None);

    let mut game_camera = OrthographicCameraBundle::new_2d();

    game_camera.orthographic_projection.scaling_mode = ScalingMode::FixedVertical;
    game_camera.orthographic_projection.scale = projection_scale;

    commands.spawn_bundle(game_camera);
    commands.spawn_bundle(UiCameraBundle::default());
}

fn app_controls(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    button_press: Res<ButtonPress>,
    mut app_state: ResMut<State<AppState>>,
) {
    let should_go_to_menu =
        keyboard_input.just_released(KeyCode::Escape) || button_press.start_pressed;

    if should_go_to_menu && app_state.current() != &AppState::InMenu {
        app_state
            .set(AppState::InMenu)
            .expect("Could show the main menu");

        keyboard_input.reset(KeyCode::Escape);
    }
}

fn update_button_input(
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

fn window_resize(
    resize_event: Res<Events<WindowResized>>,
    window: Res<Windows>,
    mut query: Query<&mut OrthographicProjection, With<Camera2d>>,
) {
    let mut reader = resize_event.get_reader();
    for event in reader.iter(&resize_event) {
        for mut projection in query.iter_mut() {
            let window = window.primary();
            let projection_scale = window_to_projection_scale(window, Some(event.height));

            projection.scale = projection_scale;
        }
    }
}

fn window_to_projection_scale(window: &Window, height_override: Option<f32>) -> f32 {
    let height = if window.mode() == WindowMode::Windowed {
        height_override.unwrap_or_else(|| window.requested_height())
    } else {
        height_override.unwrap_or_else(|| window.height() as f32)
    };

    height / 2.0
}
