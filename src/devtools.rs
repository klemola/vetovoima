use std::env;

use bevy::{
    app::PluginGroupBuilder,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_rapier2d::prelude::*;

use crate::{
    app::{UiConfig, VetovoimaColor},
    game::Player,
    simulation::GravitySource,
};

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct GravityText;

#[derive(Component)]
struct PlayerText;

#[derive(Default)]
struct DebugOutputPlugin;

impl Plugin for DebugOutputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, debug_setup);
        app.add_systems(
            Update,
            (
                fps_text_update,
                gravity_debug_text_update,
                player_text_update,
            ),
        );
    }
}

pub struct DevTools;

impl PluginGroup for DevTools {
    fn build(self) -> PluginGroupBuilder {
        match env::var("DEV_TOOLS") {
            Result::Ok(value) if value == *"1" => PluginGroupBuilder::start::<Self>()
                .add(RapierDebugRenderPlugin::default())
                .add(FrameTimeDiagnosticsPlugin::default())
                .add(DebugOutputPlugin::default()),

            _ => PluginGroupBuilder::start::<Self>(),
        }
    }
}

fn debug_setup(mut commands: Commands, asset_server: Res<AssetServer>, ui_config: Res<UiConfig>) {
    let font = asset_server.load(ui_config.font_filename);

    commands
        .spawn((
            Text::new("FPS "),
            TextFont {
                font: font.clone(),
                font_size: ui_config.font_size_body_small,
                ..Default::default()
            },
            TextColor(VetovoimaColor::WHITEISH),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(58.0),
                left: Val::Px(10.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextColor(VetovoimaColor::YELLOWISH),
            FpsText,
        ));

    commands
        .spawn((
            Text::new("Gravity scale "),
            TextFont {
                font: font.clone(),
                font_size: ui_config.font_size_body_small,
                ..Default::default()
            },
            TextColor(VetovoimaColor::WHITEISH),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(34.0),
                left: Val::Px(10.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextColor(VetovoimaColor::REDDISH),
            GravityText,
        ));

    commands
        .spawn((
            Text::new("Player velocity "),
            TextFont {
                font: font.clone(),
                font_size: ui_config.font_size_body_small,
                ..Default::default()
            },
            TextColor(VetovoimaColor::WHITEISH),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextColor(VetovoimaColor::REDDISH),
            PlayerText,
        ));
}

fn fps_text_update(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_text_query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut span in fps_text_query.iter_mut() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                **span = format!("{:.2}", average);
            }
        }
    }
}

fn gravity_debug_text_update(
    mut gravity_text_query: Query<&mut TextSpan, With<GravityText>>,
    gravity_source: ResMut<GravitySource>,
) {
    for mut span in gravity_text_query.iter_mut() {
        **span = format!("{:.2}", gravity_source.force);
    }
}

fn player_text_update(
    velocity_query: Query<&Velocity, With<Player>>,
    mut player_text_query: Query<&mut TextSpan, With<PlayerText>>,
) {
    match velocity_query.get_single() {
        Err(_) => {}
        Ok(velocity) => {
            let mut span = player_text_query.single_mut();

            **span = format!(
                "[{:6.1},{:6.1}] / {:4.1}",
                velocity.linvel.x, velocity.linvel.y, velocity.angvel
            );
        }
    }
}
