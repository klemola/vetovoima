use rand::{
    distributions::{Distribution, Standard},
    prelude::*,
    Rng,
};
use std::env;
use std::f32::consts::PI;

use bevy::app::{AppExit, PluginGroupBuilder};
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::WindowMode,
};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct GravitySource {
    force: f32,
    cycle: Attraction,
    auto_cycle: bool,
}

enum Attraction {
    Positive,
    Negative,
}

#[derive(Component)]
pub struct Attractable;

#[derive(Component)]
pub struct Player;

// UI tag components

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct GravityText;

#[derive(Component)]
struct PlayerText;

#[derive(Component)]
struct ExitButton;

#[derive(Component)]
struct AutoCycleButton;

// Config

const PIXELS_PER_METER: f32 = 20.0;
const WORLD_RADIUS_METERS: f32 = 24.0;
const GRAVITY_SOURCE_RADIUS_METERS: f32 = 2.5;
const PLAYER_WIDTH_METERS: f32 = 0.8;
const PLAYER_HEIGHT_METERS: f32 = 1.8;

const PLAYER_MAX_FORWARD_VELOCITY: f32 = 64.0;
const PLAYER_SLOW_DOWN_VELOCITY: f32 = -200.0;
const PLAYER_MAX_ANGULAR_VELOCITY: f32 = 90.0;

const GRAVITY_AUTO_CYCLE_ENABLED_DEFAULT: bool = false;
const GRAVITY_FORCE_SCALE: f32 = 12_000.0 * GRAVITY_SOURCE_RADIUS_METERS;
const MAX_GRAVITY_FORCE: f32 = 1.0;
const MIN_GRAVITY_FORCE: f32 = -MAX_GRAVITY_FORCE;
const INITIAL_GRAVITY_FORCE: f32 = MAX_GRAVITY_FORCE;

const BUTTON_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);
const BUTTON_COLOR_HOVER: Color = Color::rgb(0.25, 0.25, 0.25);
const BUTTON_ACTIVE_COLOR: Color = Color::rgb(0.35, 0.75, 0.35);
static EXIT_BUTTON_LABEL: &str = "Exit (ESC)";
static AUTO_CYCLE_BUTTON_LABEL: &str = "Cycle gravity (G)";

fn main() {
    assert!(MAX_GRAVITY_FORCE > MIN_GRAVITY_FORCE);
    assert!(GRAVITY_FORCE_SCALE > 0.0);
    assert!(
        INITIAL_GRAVITY_FORCE <= MAX_GRAVITY_FORCE && INITIAL_GRAVITY_FORCE >= MIN_GRAVITY_FORCE
    );

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: "Vetovoima".to_string(),
            mode: WindowMode::Fullscreen,
            ..default()
        })
        .insert_resource(GravitySource {
            force: INITIAL_GRAVITY_FORCE,
            cycle: Attraction::Negative,
            auto_cycle: GRAVITY_AUTO_CYCLE_ENABLED_DEFAULT,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DevTools)
        .add_plugin(ShapePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .add_startup_system(simulation_setup)
        .add_startup_system(ui_setup)
        .add_system(keyboard_input_effects)
        .add_system(update_gravity)
        .add_system(apply_forces)
        .add_system(update_player_velocity)
        .add_system(exit_button_event)
        .add_system(auto_cycle_button_event)
        .run();
}

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

struct DevTools;

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
                    top: Val::Px(56.0),
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
                    top: Val::Px(80.0),
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
                    top: Val::Px(104.0),
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

fn simulation_setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // World (the hollow circle that bounds the simulation)
    let world_radius_pixels = WORLD_RADIUS_METERS * PIXELS_PER_METER;
    let world_thickness = 1.0;
    let world_shape = shapes::Circle {
        radius: world_radius_pixels,
        center: Vec2::ZERO,
    };
    let world_vertices: Vec<Vec2> = (0..=360)
        .map(|a: i32| {
            let a_rad: f32 = a as f32 * (PI / 180.0);
            let r = world_radius_pixels - world_thickness;
            let x = r * a_rad.cos();
            let y = r * a_rad.sin();

            Vec2::new(x, y)
        })
        .collect();

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &world_shape,
            DrawMode::Stroke(StrokeMode {
                color: Color::BLUE,
                options: StrokeOptions::DEFAULT,
            }),
            Transform::default(),
        ))
        .insert(Collider::polyline(world_vertices, None));

    // Gravity source (as a world object)
    let gravity_source_radius_pixels = GRAVITY_SOURCE_RADIUS_METERS * PIXELS_PER_METER;
    let gravity_source_shape = shapes::Circle {
        radius: gravity_source_radius_pixels,
        center: Vec2::ZERO,
    };

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &gravity_source_shape,
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::WHITE)),
            Transform::default(),
        ))
        .insert(RigidBody::Fixed)
        .insert(Collider::ball(gravity_source_radius_pixels))
        .insert(Restitution::coefficient(0.1));

    // "Player"
    let player_extent_x = PLAYER_WIDTH_METERS * PIXELS_PER_METER;
    let player_extent_y = PLAYER_HEIGHT_METERS * PIXELS_PER_METER;

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shapes::Rectangle {
                extents: Vec2::new(player_extent_x, player_extent_y),
                origin: RectangleOrigin::Center,
            },
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::YELLOW)),
            Transform::from_translation(Vec3::new(0.0, -100.0, 0.0)),
        ))
        .insert(Attractable {})
        .insert(Player {})
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(
            player_extent_x / 2.0,
            player_extent_y / 2.0,
        ))
        .insert(MassProperties {
            local_center_of_mass: Vec2::new(0.0, -player_extent_y),
            mass: 100.0,
            principal_inertia: 0.2,
        })
        .insert(Restitution::coefficient(0.1))
        .insert(GravityScale(0.0))
        .insert(ExternalForce {
            force: Vec2::ZERO,
            torque: 0.0,
        })
        .insert(Velocity {
            linvel: Vec2::ZERO,
            angvel: 0.0,
        });

    spawn_objects(&mut commands);
}

enum ObjectKind {
    Rectangle,
    Hexagon,
    Circle,
}

impl Distribution<ObjectKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ObjectKind {
        let distance: f32 = rng.gen();

        if distance > 0.8 {
            ObjectKind::Circle
        } else if distance < 0.25 {
            ObjectKind::Hexagon
        } else {
            ObjectKind::Rectangle
        }
    }
}

enum ObjectDensity {
    Light,
    Medium,
    Heavy,
}

impl Distribution<ObjectDensity> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ObjectDensity {
        let distance: f32 = rng.gen();

        if distance > 0.85 {
            ObjectDensity::Heavy
        } else if distance < 0.3 {
            ObjectDensity::Light
        } else {
            ObjectDensity::Medium
        }
    }
}

fn spawn_objects(commands: &mut Commands) {
    let max_objects = 20;
    let full_turn_radians = 2.0 * PI;

    for n in 1..=max_objects {
        let object_kind: ObjectKind = rand::random();
        let object_density: ObjectDensity = rand::random();
        let range = match object_density {
            ObjectDensity::Light => 0.2..=0.5,
            ObjectDensity::Medium => 0.4..=0.8,
            ObjectDensity::Heavy => 0.8..=0.9,
        };
        let distance_from_center_meters: f32 = thread_rng().gen_range(range) * WORLD_RADIUS_METERS;
        let base_x = distance_from_center_meters * PIXELS_PER_METER;
        let angle = (full_turn_radians / max_objects as f32) * n as f32;
        let mut transform = Transform::from_translation(Vec3::new(base_x, 0.0, 0.0));

        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_z(angle));
        spawn_object(commands, object_kind, object_density, transform);
    }
}

fn spawn_object(
    commands: &mut Commands,
    kind: ObjectKind,
    density: ObjectDensity,
    transform: Transform,
) {
    let (density_value, base_scale_factor, color) = match density {
        ObjectDensity::Light => (0.75, 1.0, Color::GRAY),
        ObjectDensity::Medium => (1.0, 2.0, Color::RED),
        ObjectDensity::Heavy => (10.0, 3.2, Color::BLUE),
    };
    let scale_variation: f32 = thread_rng().gen_range(-0.2..0.4);
    let scale_factor = (base_scale_factor + (base_scale_factor * scale_variation)).max(1.0);
    let (shape_bundle, collider, restitution_coefficient) = match kind {
        ObjectKind::Rectangle => rectangle_props(transform, scale_factor, color),
        ObjectKind::Hexagon => hexagon_props(transform, scale_factor, color),
        ObjectKind::Circle => circle_props(transform, scale_factor, color),
    };

    commands
        .spawn_bundle(shape_bundle)
        .insert(Attractable {})
        .insert(RigidBody::Dynamic)
        .insert(collider)
        .insert(ColliderMassProperties::Density(density_value))
        .insert(Restitution::coefficient(restitution_coefficient))
        .insert(GravityScale(0.0))
        .insert(ExternalForce {
            force: Vec2::ZERO,
            torque: 0.0,
        });
}

fn rectangle_props(
    transform: Transform,
    scale_factor: f32,
    color: Color,
) -> (ShapeBundle, Collider, f32) {
    let extent = scale_factor * PIXELS_PER_METER;

    let shape = shapes::Rectangle {
        extents: Vec2::new(extent, extent),
        origin: RectangleOrigin::Center,
    };

    let shape_bundle = GeometryBuilder::build_as(
        &shape,
        DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(color)),
        transform,
    );

    let collider = Collider::cuboid(extent / 2.0, extent / 2.0);

    (shape_bundle, collider, 0.1)
}

fn hexagon_props(
    transform: Transform,
    scale_factor: f32,
    color: Color,
) -> (ShapeBundle, Collider, f32) {
    let radius: f32 = 0.5 * scale_factor * PIXELS_PER_METER;
    let step_angle = PI / 3.0;
    let hexagon_vertices = vec![
        Vec2::new(radius, 0.0),
        Vec2::new(step_angle.cos() * radius, step_angle.sin() * radius),
        Vec2::new(
            (step_angle * 2.0).cos() * radius,
            (step_angle * 2.0).sin() * radius,
        ),
        Vec2::new(-radius, 0.0),
        Vec2::new(
            -(step_angle * 2.0).cos() * radius,
            -(step_angle * 2.0).sin() * radius,
        ),
        Vec2::new(-step_angle.cos() * radius, -step_angle.sin() * radius),
    ];

    let shape = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(radius),
        ..shapes::RegularPolygon::default()
    };

    let shape_bundle = GeometryBuilder::build_as(
        &shape,
        DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(color)),
        transform,
    );

    let collider = Collider::convex_hull(&hexagon_vertices).unwrap_or(Collider::ball(radius));

    (shape_bundle, collider, 0.2)
}

fn circle_props(
    transform: Transform,
    scale_factor: f32,
    color: Color,
) -> (ShapeBundle, Collider, f32) {
    let radius: f32 = 0.5 * scale_factor * PIXELS_PER_METER;

    let shape = shapes::Circle {
        radius: radius,
        center: Vec2::ZERO,
    };

    let shape_bundle = GeometryBuilder::build_as(
        &shape,
        DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(color)),
        transform,
    );

    let collider = Collider::ball(radius);

    (shape_bundle, collider, 1.0)
}

fn ui_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("VT323-Regular.ttf");
    let font_size = 24.0;

    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(36.0)),
                position: Rect {
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
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
                        font: font.clone(),
                        font_size: font_size,
                        color: Color::WHITE,
                    },
                    Default::default(),
                ),
                ..default()
            });
        })
        .insert(ExitButton);

    let auto_cycle_button_color = if GRAVITY_AUTO_CYCLE_ENABLED_DEFAULT {
        BUTTON_ACTIVE_COLOR
    } else {
        BUTTON_COLOR
    };

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(190.0), Val::Px(36.0)),
                position: Rect {
                    top: Val::Px(10.0),
                    left: Val::Px(140.0),
                    ..default()
                },
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: auto_cycle_button_color.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    AUTO_CYCLE_BUTTON_LABEL,
                    TextStyle {
                        font: font.clone(),
                        font_size: font_size,
                        color: Color::WHITE,
                    },
                    Default::default(),
                ),
                ..default()
            });
        })
        .insert(AutoCycleButton);
}

// Simulation systems

fn update_gravity(
    mut gravity_source: ResMut<GravitySource>,
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let force_change = if gravity_source.auto_cycle {
        let increment = timer.delta_seconds() / 2.0;

        match gravity_source.cycle {
            Attraction::Positive => -increment,
            Attraction::Negative => increment,
        }
    } else {
        let increment = 0.04;

        if keyboard_input.pressed(KeyCode::Up) {
            -increment
        } else if keyboard_input.pressed(KeyCode::Down) {
            increment
        } else {
            0.0
        }
    };

    gravity_source.force += force_change;

    if gravity_source.force >= MAX_GRAVITY_FORCE {
        // Enforce force upper limit
        gravity_source.force = MAX_GRAVITY_FORCE;
        gravity_source.cycle = Attraction::Positive;
    } else if gravity_source.force <= MIN_GRAVITY_FORCE {
        // Enforce force lower limit
        gravity_source.force = MIN_GRAVITY_FORCE;
        gravity_source.cycle = Attraction::Negative;
    }
}

fn update_player_velocity(
    mut velocities: Query<(&mut Velocity, &Transform), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    timer: Res<Time>,
) {
    let (mut vel, transform) = velocities.single_mut();

    let forward = transform.local_x();
    let forward_dir = Vec2::new(forward.x, forward.y);
    let current_velocity = (vel.linvel * forward_dir).length();
    let negative_velocity = (vel.linvel.normalize() * -forward_dir).length();
    let intensity = if keyboard_input.pressed(KeyCode::Left) && negative_velocity == 0.0 {
        // Slow down until the player halts
        PLAYER_SLOW_DOWN_VELOCITY
    } else if keyboard_input.pressed(KeyCode::Right)
        && current_velocity < PLAYER_MAX_FORWARD_VELOCITY
    {
        // Accelerate in the forward direction
        PLAYER_MAX_FORWARD_VELOCITY
    } else {
        0.0
    };
    let player_control_force = forward_dir * intensity;

    let translation_2d: Vec2 = Vec2::new(transform.translation.x, transform.translation.y);
    let dir_to_gravity_force = translation_2d.normalize();
    let dot = forward_dir.dot(dir_to_gravity_force);
    // maintain a right angle between player movement direction and gravity source direction
    // TODO: slow down when close to the target (0 dot product)
    let angular_velocity = if dot > 0.05 {
        PLAYER_MAX_ANGULAR_VELOCITY
    } else if dot < -0.05 {
        -PLAYER_MAX_ANGULAR_VELOCITY
    } else {
        0.0
    };

    vel.linvel += player_control_force * timer.delta_seconds();
    vel.angvel = (angular_velocity * timer.delta_seconds())
        .clamp(-PLAYER_MAX_ANGULAR_VELOCITY, PLAYER_MAX_ANGULAR_VELOCITY);
}

fn apply_forces(
    mut ext_forces: Query<(&mut ExternalForce, &Transform), With<Attractable>>,
    gravity_source: ResMut<GravitySource>,
) {
    for (mut ext_force, transform) in ext_forces.iter_mut() {
        let translation_2d: Vec2 = Vec2::new(transform.translation.x, transform.translation.y);

        let force_dir = translation_2d.normalize();
        let base_force = force_dir * gravity_source.force * GRAVITY_FORCE_SCALE;
        let gravity_force = base_force / translation_2d.length();
        ext_force.force = gravity_force;
    }
}

// Input systems

fn keyboard_input_effects(
    keyboard_input: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut gravity_source: ResMut<GravitySource>,
) {
    if keyboard_input.just_released(KeyCode::Escape) {
        exit.send(AppExit);
    }

    if keyboard_input.just_released(KeyCode::G) {
        gravity_source.auto_cycle = !gravity_source.auto_cycle;
    }
}

// UI systems

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
    let velocity = velocity_query.single();
    let mut text = text_query.single_mut();

    text.sections[1].value = format!(
        "[{:6.1},{:6.1}] / {:4.1}",
        velocity.linvel.x, velocity.linvel.y, velocity.angvel
    );
}

fn exit_button_event(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<ExitButton>),
    >,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                exit.send(AppExit);
                *color = BUTTON_ACTIVE_COLOR.into();
            }
            Interaction::Hovered => {
                *color = BUTTON_COLOR_HOVER.into();
            }
            Interaction::None => {
                *color = BUTTON_COLOR.into();
            }
        }
    }
}

fn auto_cycle_button_event(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<AutoCycleButton>),
    >,
    mut gravity_source: ResMut<GravitySource>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        let color_at_rest = if gravity_source.auto_cycle {
            BUTTON_ACTIVE_COLOR
        } else {
            BUTTON_COLOR
        };

        match *interaction {
            Interaction::Clicked => {
                *color = color_at_rest.into();
                gravity_source.auto_cycle = !gravity_source.auto_cycle;
            }
            Interaction::Hovered => {
                *color = if gravity_source.auto_cycle {
                    BUTTON_ACTIVE_COLOR.into()
                } else {
                    BUTTON_COLOR_HOVER.into()
                };
            }
            Interaction::None => {
                *color = color_at_rest.into();
            }
        }
    }
}
