use std::env;

use bevy::{
    app::PluginGroupBuilder,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_rapier2d::prelude::*;

use crate::game::Player;
use crate::simulation::GravitySource;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct GravityText;

#[derive(Component)]
struct PlayerText;

struct DebugOutputPlugin;

impl Plugin for DebugOutputPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(debug_setup)
            .add_system(fps_text_update)
            .add_system(gravity_debug_text_update)
            .add_system(player_text_update);
    }
}

impl Default for DebugOutputPlugin {
    fn default() -> Self {
        Self {}
    }
}

pub struct DevTools;

impl PluginGroup for DevTools {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        match env::var("DEV_TOOLS") {
            Result::Ok(value) if value == "1".to_string() => group
                .add(RapierDebugRenderPlugin::default())
                .add(FrameTimeDiagnosticsPlugin::default())
                .add(DebugOutputPlugin::default()),

            _ => group,
        };
    }
}

fn debug_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("VT323-Regular.ttf");
    let font_size = 24.0;

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS ".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size,
                            color: Color::YELLOW,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        })
        .insert(FpsText);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(34.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Gravity scale ".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size,
                            color: Color::RED,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        })
        .insert(GravityText);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(58.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Player velocity ".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size,
                            color: Color::RED,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        })
        .insert(PlayerText);
}

fn fps_text_update(diagnostics: Res<Diagnostics>, mut fps_query: Query<&mut Text, With<FpsText>>) {
    for mut text in fps_query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.sections[1].value = format!("{:.2}", average);
            }
        }
    }
}

fn gravity_debug_text_update(
    mut gravity_query: Query<&mut Text, With<GravityText>>,
    gravity_source: ResMut<GravitySource>,
) {
    for mut text in gravity_query.iter_mut() {
        text.sections[1].value = format!("{:.2}", gravity_source.force);
    }
}

fn player_text_update(
    velocity_query: Query<&Velocity, With<Player>>,
    mut text_query: Query<&mut Text, With<PlayerText>>,
) {
    match velocity_query.get_single() {
        Err(_) => {}
        Ok(velocity) => {
            let mut text = text_query.single_mut();

            text.sections[1].value = format!(
                "[{:6.1},{:6.1}] / {:4.1}",
                velocity.linvel.x, velocity.linvel.y, velocity.angvel
            );
        }
    }
}
