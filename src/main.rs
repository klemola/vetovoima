mod app;
mod devtools;
mod game;
mod game_over;
mod main_menu;
mod simulation;
mod sounds;

use bevy::{
    ecs::event::Events,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::camera::ScalingMode,
    window::{PrimaryWindow, WindowMode, WindowResized, WindowResolution},
};
use bevy_rapier2d::plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin};

use app::{
    get_config_or_default, AppState, ButtonPress, UiConfig, VetovoimaColor, APP_NAME,
    PIXELS_PER_METER,
};
use devtools::DevTools;
use game::GamePlugin;
use game_over::GameOverPlugin;
use main_menu::MainMenuPlugin;
use simulation::SimulationPlugin;
use sounds::SoundsPlugin;

fn main() {
    let vv_config = get_config_or_default();
    let window_resolution = match (
        vv_config.window_width_pixels,
        vv_config.window_height_pixels,
    ) {
        // Apply the user-defined resolution only if both width and height are specified
        (Some(width), Some(height)) => WindowResolution::new(width as f32, height as f32),
        _ => WindowResolution::default(),
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: APP_NAME.into(),
                mode: vv_config.window_mode,
                resolution: window_resolution,
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            SoundsPlugin,
            MainMenuPlugin,
            SimulationPlugin,
            GamePlugin,
            GameOverPlugin,
            DevTools,
        ))
        .insert_resource(ClearColor(VetovoimaColor::BLACKISH))
        .insert_resource(ButtonPress::default())
        .insert_resource(UiConfig::default())
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::Init), app_setup)
        .add_systems(
            Update,
            (
                app_controls,
                keyboard_input,
                window_resize,
                transition_to_in_menu.run_if(in_state(AppState::Init)),
            ),
        )
        .run();
}

fn app_setup(mut commands: Commands, primary_window: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };
    let (projection_scale, _window_height) = window_to_projection_scale(window, None);

    let game_camera = Camera2d::default();

    commands.spawn((
        game_camera,
        Msaa::Sample4,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 2.0,
            },
            scale: projection_scale,
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn(RapierConfiguration {
        gravity: Vec2::ZERO,
        ..RapierConfiguration::new(PIXELS_PER_METER)
    });
}

fn app_controls(
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    button_press: Res<ButtonPress>,
    app_state: ResMut<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let should_go_to_menu =
        keyboard_input.just_released(KeyCode::Escape) || button_press.start_pressed;

    if should_go_to_menu && *app_state.get() != AppState::InMenu {
        next_app_state.set(AppState::InMenu);
        keyboard_input.reset(KeyCode::Escape);
    }
}

fn transition_to_in_menu(mut app_state: ResMut<NextState<AppState>>) {
    app_state.set(AppState::InMenu);
}

fn keyboard_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut button_press: ResMut<ButtonPress>,
) {
    for event in keyboard_events.read() {
        let is_pressed = ButtonState::is_pressed(&event.state);

        match event.key_code {
            KeyCode::ArrowUp => button_press.up_pressed = is_pressed,
            KeyCode::ArrowDown => button_press.down_pressed = is_pressed,
            KeyCode::ArrowLeft => button_press.left_pressed = is_pressed,
            KeyCode::ArrowRight => button_press.right_pressed = is_pressed,
            KeyCode::Enter => button_press.main_control_pressed = is_pressed,
            KeyCode::Escape => button_press.select_pressed = is_pressed,

            _ => (),
        }
    }
}

fn window_resize(
    resize_event: Res<Events<WindowResized>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut OrthographicProjection, With<Camera2d>>,
    mut ui_config: ResMut<UiConfig>,
) {
    let mut reader = resize_event.get_cursor();
    for event in reader.read(&resize_event) {
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
            projection.scaling_mode = ScalingMode::FixedVertical {
                viewport_height: scale_ratio * 1.1,
            };
            *ui_config = scaled_ui_config;
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
