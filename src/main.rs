mod app;
mod devtools;
mod game;
mod game_over;
mod main_menu;
mod simulation;

use bevy::{prelude::*, window::WindowMode};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};

use app::{AppState, APP_NAME, PIXELS_PER_METER};
use devtools::DevTools;
use game::GamePlugin;
use game_over::GameOverPlugin;
use main_menu::MainMenuPlugin;
use simulation::SimulationPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: APP_NAME.into(),
            mode: WindowMode::Fullscreen,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .add_plugin(MainMenuPlugin)
        .add_plugin(SimulationPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(GameOverPlugin)
        .add_plugins(DevTools)
        .add_state(AppState::InMenu)
        .add_startup_system(app_setup)
        .add_system(app_controls)
        .run();
}

fn app_setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

pub fn app_controls(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut app_state: ResMut<State<AppState>>,
) {
    if keyboard_input.just_released(KeyCode::Escape) {
        if app_state.current() != &AppState::InMenu {
            app_state
                .set(AppState::InMenu)
                .expect("Could show the main menu");

            keyboard_input.reset(KeyCode::Escape);
        }
    }
}
