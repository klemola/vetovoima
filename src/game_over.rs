use bevy::prelude::*;
use std::time::Duration;

use crate::app::{AppState, VetovoimaColor};

const GAME_OVER_SCREEN_SHOW_DURATION_SECONDS: u64 = 5;

#[derive(Component, Resource)]
struct GameOverScreen(Timer);

#[derive(Component)]
struct GameOverTitle;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::GameOver).with_system(gameover_screen_setup),
        )
        .add_system_set(
            SystemSet::on_update(AppState::GameOver).with_system(gameover_screen_update),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::GameOver).with_system(gameover_screen_cleanup),
        );
    }
}

fn gameover_screen_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameOverScreen(Timer::new(
        Duration::from_secs(GAME_OVER_SCREEN_SHOW_DURATION_SECONDS),
        TimerMode::Once,
    )));

    let font = asset_server.load("VT323-Regular.ttf");
    let font_size = 100.0;

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: VetovoimaColor::BLACKISH.into(),
            ..default()
        })
        .with_children(|container| {
            container.spawn(TextBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    ..default()
                },
                text: Text::from_section(
                    "GAME OVER",
                    TextStyle {
                        font,
                        font_size,
                        color: VetovoimaColor::REDDISH,
                    },
                ),
                ..default()
            });
        })
        .insert(GameOverTitle);
}

fn gameover_screen_cleanup(
    mut commands: Commands,
    title_query: Query<Entity, With<GameOverTitle>>,
) {
    commands.remove_resource::<GameOverScreen>();

    for object in title_query.iter() {
        commands.entity(object).despawn_recursive();
    }
}

fn gameover_screen_update(
    mut game_over_screen: ResMut<GameOverScreen>,
    mut app_state: ResMut<State<AppState>>,
    time: Res<Time>,
) {
    game_over_screen.0.tick(time.delta());

    if game_over_screen.0.finished() {
        app_state
            .set(AppState::InMenu)
            .expect("Could not transition to the menu");
    }
}
