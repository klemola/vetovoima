use rand::{prelude::*, seq::IteratorRandom, Rng};
use rand_distr::{Distribution, Normal, Standard};
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    InMenu,
    LoadingLevel,
    InGame,
    GameOver,
    ObserveSimulation,
}

#[derive(Component)]
enum MenuButton {
    NewGame,
    Exit,
    SimulationMode,
}

#[derive(Component)]
struct World {
    vertices: Vec<Vec2>,
    vertices_inner_edge: Vec<Vec2>,
}

#[derive(Component)]
struct GravitySource {
    force: f32,
    cycle: Attraction,
    auto_cycle: bool,
}

enum Attraction {
    Positive,
    Negative,
}

#[derive(Component)]
struct GameObject;

#[derive(Component)]
struct Attractable;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Flag;

// UI tag components

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct GravityText;

#[derive(Component)]
struct PlayerText;

// Config

const PIXELS_PER_METER: f32 = 16.0;
const WORLD_RADIUS_METERS: f32 = 28.0;
const GRAVITY_SOURCE_RADIUS_METERS: f32 = 2.5;
const PLAYER_WIDTH_METERS: f32 = 0.8;
const PLAYER_HEIGHT_METERS: f32 = 1.8;
const FLAG_WIDTH_METERS: f32 = 0.4;
const FLAG_HEIGHT_METERS: f32 = 3.0;

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
static APP_NAME: &str = "vetovoima";
static NEW_GAME_BUTTON_LABEL: &str = "New game";
static EXIT_BUTTON_LABEL: &str = "Exit";

fn main() {
    assert!(MAX_GRAVITY_FORCE > MIN_GRAVITY_FORCE);
    assert!(GRAVITY_FORCE_SCALE > 0.0);
    assert!(
        INITIAL_GRAVITY_FORCE <= MAX_GRAVITY_FORCE && INITIAL_GRAVITY_FORCE >= MIN_GRAVITY_FORCE
    );

    let world: World = create_world();

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: APP_NAME.into(),
            mode: WindowMode::Fullscreen,
            ..default()
        })
        .insert_resource(world)
        .insert_resource(GravitySource {
            force: INITIAL_GRAVITY_FORCE,
            cycle: Attraction::Negative,
            auto_cycle: GRAVITY_AUTO_CYCLE_ENABLED_DEFAULT,
        })
        .add_state(AppState::InMenu)
        .add_plugins(DefaultPlugins)
        .add_plugins(DevTools)
        .add_plugin(ShapePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .add_startup_system(simulation_setup)
        .add_startup_system(ui_setup)
        .add_system_set(SystemSet::on_enter(AppState::InMenu).with_system(show_menu))
        .add_system_set(
            SystemSet::on_update(AppState::InMenu)
                .with_system(menu_button_state)
                .with_system(menu_button_event),
        )
        .add_system_set(SystemSet::on_exit(AppState::InMenu).with_system(hide_menu))
        .add_system_set(SystemSet::on_enter(AppState::LoadingLevel).with_system(game_setup))
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(update_gravity)
                .with_system(apply_forces)
                .with_system(update_player_velocity)
                .with_system(check_goal_reached),
        )
        .add_system(main_controls)
        .run();
}

fn create_world() -> World {
    // World (the hollow circle that bounds the simulation)
    let radius_pixels = WORLD_RADIUS_METERS * PIXELS_PER_METER;
    // the outer edge of the circle polygon
    let outer_circle_steps = 180;
    let vertices_outer: Vec<Vec2> = (0..=outer_circle_steps)
        .map(|step: i32| {
            let step_multiplier = 360 / outer_circle_steps;
            let a = (step * step_multiplier) as f32;
            let a_rad: f32 = a * (PI / 180.0);
            let r = radius_pixels;
            let x = r * a_rad.cos();
            let y = r * a_rad.sin();

            Vec2::new(x, y)
        })
        .collect();
    // the inner edge of the circle polygon (with some variation)
    let inner_circle_steps = 180;
    let vertices_inner_edge: Vec<Vec2> = (0..=inner_circle_steps)
        .map(|step: i32| {
            let mean = 1.6 * PIXELS_PER_METER;
            let variation = if step > 0 && step < inner_circle_steps {
                let std_deviation = 2.2;
                let normal_distribution = Normal::new(mean, std_deviation).unwrap();
                normal_distribution.sample(&mut rand::thread_rng())
            } else {
                mean
            };
            let step_multiplier = 360 / inner_circle_steps;
            let a = (step * step_multiplier) as f32;
            let a_rad: f32 = a * (PI / 180.0);
            let r = radius_pixels - variation;
            let x = r * a_rad.cos();
            let y = r * a_rad.sin();

            Vec2::new(x, y)
        })
        .collect();

    World {
        vertices: vec![vertices_inner_edge.clone(), vertices_outer.clone()].concat(),
        vertices_inner_edge,
    }
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

fn simulation_setup(mut commands: Commands, world: Res<World>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let world_shape = &shapes::Polygon {
        points: world.vertices.clone(),
        closed: true,
    };

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            world_shape,
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::WHITE)),
            Transform::default(),
        ))
        .insert(Collider::polyline(world.vertices_inner_edge.clone(), None));

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
}

fn ui_setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

fn show_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("VT323-Regular.ttf");
    let font_size = 64.0;

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                padding: Rect::all(Val::Px(10.0)),
                ..Default::default()
            },
            color: Color::BLACK.into(),
            ..Default::default()
        })
        .insert(MainMenu)
        .with_children(|menu_node| {
            menu_node
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(600.0), Val::Px(120.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(20.0)),
                        ..default()
                    },
                    color: Color::BLACK.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            APP_NAME,
                            TextStyle {
                                font: font.clone(),
                                font_size: font_size * 1.6,
                                color: Color::WHITE,
                            },
                            Default::default(),
                        ),
                        ..default()
                    });
                });

            menu_node
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(400.0), Val::Px(80.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(10.0)),
                        ..default()
                    },
                    color: BUTTON_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            NEW_GAME_BUTTON_LABEL,
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
                .insert(MenuButton::NewGame);

            menu_node
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(400.0), Val::Px(80.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(10.0)),
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
                .insert(MenuButton::Exit);
        });
}

fn hide_menu(mut commands: Commands, menu: Query<Entity, With<MainMenu>>) {
    let menu = menu.get_single().expect("Could not hide the menu");
    commands.entity(menu).despawn_recursive();
}

fn game_setup(
    mut commands: Commands,
    game_object_query: Query<Entity, With<GameObject>>,
    mut app_state: ResMut<State<AppState>>,
    world: Res<World>,
) {
    for object in game_object_query.iter() {
        commands.entity(object).despawn();
    }

    spawn_objects(&mut commands);
    spawn_player_and_and_goal(&mut commands, world);

    app_state
        .set(AppState::InGame)
        .expect("Tried to enter the game from loading, but failed");
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
    let max_objects = 16;
    let full_turn_radians = 2.0 * PI;

    for n in 1..=max_objects {
        let object_kind: ObjectKind = rand::random();
        let object_density: ObjectDensity = rand::random();
        let range = match object_density {
            ObjectDensity::Light => 0.2..=0.4,
            ObjectDensity::Medium => 0.4..=0.6,
            ObjectDensity::Heavy => 0.8..=0.8,
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
        ObjectDensity::Light => (0.75, 1.0, Color::PINK),
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
        .insert(GameObject)
        .insert(Attractable)
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

fn spawn_player_and_and_goal(commands: &mut Commands, world: Res<World>) {
    // Flag (goal)
    let flag_extent_x = FLAG_WIDTH_METERS * PIXELS_PER_METER;
    let flag_extent_y = FLAG_HEIGHT_METERS * PIXELS_PER_METER;
    // a point somewhere along the inner edge of the world (the ground)
    let flag_anchor = world
        .vertices_inner_edge
        .iter()
        .choose(&mut thread_rng())
        .unwrap_or(&Vec2::ZERO);
    let flag_transform = stand_upright_at_anchor(&flag_anchor, flag_extent_y);

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shapes::Rectangle {
                extents: Vec2::new(flag_extent_x, flag_extent_y),
                origin: RectangleOrigin::Center,
            },
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::ORANGE)),
            flag_transform,
        ))
        .insert(GameObject)
        .insert(Flag)
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(flag_extent_x / 2.0, flag_extent_y / 2.0));

    // "Player"
    let player_extent_x = PLAYER_WIDTH_METERS * PIXELS_PER_METER;
    let player_extent_y = PLAYER_HEIGHT_METERS * PIXELS_PER_METER;
    let min_player_distance_from_flag = WORLD_RADIUS_METERS * PIXELS_PER_METER * 1.8;
    let fallback_anchor = &Vec2::new(0.0, WORLD_RADIUS_METERS * PIXELS_PER_METER * -0.5);
    let player_anchor = world
        .vertices_inner_edge
        .iter()
        .find(|ground_vertex| ground_vertex.distance(*flag_anchor) > min_player_distance_from_flag)
        .unwrap_or(fallback_anchor);
    let player_transform = stand_upright_at_anchor(&player_anchor, player_extent_y);

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shapes::Rectangle {
                extents: Vec2::new(player_extent_x, player_extent_y),
                origin: RectangleOrigin::Center,
            },
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::YELLOW)),
            player_transform,
        ))
        .insert(GameObject)
        .insert(Attractable)
        .insert(Player)
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
}

fn stand_upright_at_anchor(anchor: &Vec2, object_height: f32) -> Transform {
    let dir_to_gravity_force = -anchor.normalize();
    let angle_to_gravity_force = dir_to_gravity_force.y.atan2(dir_to_gravity_force.x);
    // move the anchor towards the gravity force by half it's height to make it stick from the ground
    let anchor_aligned_with_ground = *anchor + (dir_to_gravity_force * object_height * 0.5);

    Transform::from_translation(Vec3::new(
        anchor_aligned_with_ground.x,
        anchor_aligned_with_ground.y,
        0.0,
    ))
    // the extra angle aligns positive Y (the top of the flag pole) with the gravity force
    .with_rotation(Quat::from_rotation_z(angle_to_gravity_force - (PI / 2.0)))
}

// Common systems

fn main_controls(
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

// Player systems

fn update_player_velocity(
    mut velocities: Query<(&mut Velocity, &Transform), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    timer: Res<Time>,
) {
    match velocities.get_single_mut() {
        Err(_) => {
            // The player has not been spawned yet (or the level might be changing)
        }
        Ok((mut vel, transform)) => {
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
            let dir_from_gravity_source = translation_2d.normalize();
            let dot = forward_dir.dot(dir_from_gravity_source);
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
    }
}

fn check_goal_reached(
    player_query: Query<(&Transform, &Collider), With<Player>>,
    flag_query: Query<Entity, With<Flag>>,
    mut app_state: ResMut<State<AppState>>,
    rapier_context: Res<RapierContext>,
) {
    match (player_query.get_single(), flag_query.get_single()) {
        (Ok((player_transform, player_shape)), Ok(flag_entity)) => {
            let shape_pos: Vec2 = Vec2::new(
                player_transform.translation.x,
                player_transform.translation.y,
            );
            let (_, player_angle) = player_transform.rotation.to_axis_angle();
            let groups = InteractionGroups::all();
            let flag_id = flag_entity.id();

            rapier_context.intersections_with_shape(
                shape_pos,
                player_angle,
                player_shape,
                groups,
                Some(&|entity: Entity| entity.id() == flag_id),
                |_| {
                    app_state
                        .set(AppState::LoadingLevel)
                        .expect("Could not change the level upon reaching the goal");
                    true
                },
            );
        }
        _ => (),
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

fn menu_button_state(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<MenuButton>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
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

fn menu_button_event(
    interaction_query: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut app_state: ResMut<State<AppState>>,
    mut gravity_source: ResMut<GravitySource>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, button) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => match button {
                MenuButton::NewGame => app_state
                    .set(AppState::LoadingLevel)
                    .expect("Could not start the game"),
                MenuButton::Exit => exit.send(AppExit),
                MenuButton::SimulationMode => {
                    app_state
                        .set(AppState::ObserveSimulation)
                        .expect("Could not start the observe simulation mode");
                    gravity_source.auto_cycle = true;
                }
            },

            _ => (),
        }
    }
}
