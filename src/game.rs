use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::plugin::RapierContext;
use bevy_rapier2d::prelude::*;
use rand::{prelude::*, seq::IteratorRandom, Rng};
use rand_distr::{Distribution, Normal, Standard};
use std::f32::consts::PI;
use std::time::Duration;

use crate::app::{cursor_visible, AppState, PIXELS_PER_METER};
use crate::simulation::{
    apply_forces, update_gravity, Attractable, GravitySource, GRAVITY_SOURCE_RADIUS_METERS,
};

const LEVEL_BOUNDS_RADIUS_METERS: f32 = 28.0;
const PLAYER_WIDTH_METERS: f32 = 0.8;
const PLAYER_HEIGHT_METERS: f32 = 1.8;
const FLAG_WIDTH_METERS: f32 = 0.5;
const FLAG_HEIGHT_METERS: f32 = LEVEL_BOUNDS_RADIUS_METERS / 5.0;

const PLAYER_MAX_FORWARD_VELOCITY: f32 = 64.0;
const PLAYER_SLOW_DOWN_VELOCITY: f32 = -200.0;
const PLAYER_MAX_ANGULAR_VELOCITY: f32 = 90.0;

const BASE_OBJECTS_AMOUNT: i32 = 16;
const MAX_OBJECTS_AMOUNT: i32 = 60;
const COUNTDOWN_TO_GAME_OVER_SECONDS: u64 = 30;

#[derive(Component, Clone, Debug)]
struct GameLevel {
    n: i32,
    countdown_to_game_over: Timer,
    terrain_vertices: Vec<Vec2>,
    elevation_vertices: Vec<Vec2>,
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

#[derive(Component)]
struct GameObject;

#[derive(Component)]
struct GravityRing(f32);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct Flag;

#[derive(Component)]
struct GameUIText;

#[derive(Component)]
struct GameOverCountdownText;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .add_system_set(
                SystemSet::on_enter(AppState::InitGame).with_system(clear_game_progress),
            )
            .add_system_set(SystemSet::on_enter(AppState::LoadingLevel).with_system(game_setup))
            .add_system_set(
                SystemSet::on_enter(AppState::InGame)
                    .with_system(game_ui_setup)
                    .with_system(cursor_visible::<false>),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(update_gravity)
                    .with_system(apply_forces)
                    .with_system(update_player_velocity)
                    .with_system(check_goal_reached)
                    .with_system(update_game_over_countdown)
                    .with_system(countdown_text_update)
                    .with_system(update_gravity_visuals),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::InGame)
                    .with_system(game_cleanup)
                    .with_system(game_ui_cleanup),
            );
    }
}

fn create_game_level(current_level_value: i32) -> GameLevel {
    let radius_pixels = LEVEL_BOUNDS_RADIUS_METERS * PIXELS_PER_METER;
    // the outer edge (rim) of the circle polygon
    let outer_circle_steps = 180;
    let rim_vertices: Vec<Vec2> = (0..=outer_circle_steps)
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
    // the inner edge of the circle polygon (the elevation)
    let inner_circle_steps = 180;
    let elevation_vertices: Vec<Vec2> = (0..=inner_circle_steps)
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

    GameLevel {
        n: current_level_value + 1,
        countdown_to_game_over: Timer::new(
            Duration::from_secs(COUNTDOWN_TO_GAME_OVER_SECONDS),
            false,
        ),
        terrain_vertices: vec![elevation_vertices.clone(), rim_vertices.clone()].concat(),
        elevation_vertices,
    }
}

fn clear_game_progress(mut commands: Commands, mut app_state: ResMut<State<AppState>>) {
    commands.remove_resource::<GameLevel>();
    app_state
        .set(AppState::LoadingLevel)
        .expect("Tried to load the first level, but failed");
}

fn game_cleanup(mut commands: Commands, game_object_query: Query<Entity, With<GameObject>>) {
    for object in game_object_query.iter() {
        commands.entity(object).despawn();
    }
}

fn game_setup(
    mut commands: Commands,
    mut app_state: ResMut<State<AppState>>,
    game_level: Option<Res<GameLevel>>,
    mut gravity_source: ResMut<GravitySource>,
) {
    let current_game_level_n = match game_level {
        Some(level) => level.n,
        None => 0,
    };

    let next_game_level = create_game_level(current_game_level_n);

    // Will replace the current game level with the next
    commands.insert_resource(next_game_level.clone());

    spawn_level(&mut commands, &next_game_level);
    spawn_objects(&mut commands, current_game_level_n);
    spawn_player_and_and_goal(&mut commands, &next_game_level);

    *gravity_source = GravitySource::default();

    app_state
        .set(AppState::InGame)
        .expect("Tried to enter the game from loading, but failed");
}

fn spawn_level(commands: &mut Commands, game_level: &GameLevel) {
    let level_shape = &shapes::Polygon {
        points: game_level.terrain_vertices.clone(),
        closed: true,
    };

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            level_shape,
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::WHITE)),
            Transform::default(),
        ))
        .insert(GameObject)
        .insert(Collider::polyline(
            game_level.elevation_vertices.clone(),
            None,
        ));

    // Gravity source (as a visual/physics object)
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
        .insert(GameObject)
        .insert(RigidBody::Fixed)
        .insert(Collider::ball(gravity_source_radius_pixels))
        .insert(Restitution::coefficient(0.1));

    // Gravity force visualization
    let gravity_rings_amount = 6;
    let level_bounds_radius_pixels = LEVEL_BOUNDS_RADIUS_METERS * PIXELS_PER_METER;
    let ring_frequency =
        (level_bounds_radius_pixels - gravity_source_radius_pixels) / gravity_rings_amount as f32;

    for n in 1..=gravity_rings_amount {
        let n_f = n as f32;
        let radius = ring_frequency * n_f;

        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shapes::Circle {
                    radius,
                    center: Vec2::ZERO,
                },
                DrawMode::Stroke(StrokeMode {
                    options: StrokeOptions::DEFAULT,
                    // Start with an invisible ring, the correct color will be set in the update system
                    color: Color::hsla(0.0, 1.0, 1.0, 0.0),
                }),
                Transform::default(),
            ))
            .insert(GravityRing(radius))
            .insert(GameObject);
    }
}

fn spawn_objects(commands: &mut Commands, current_game_level_n: i32) {
    let difficulty_bonus = 2 * current_game_level_n;
    let objects_amount = (BASE_OBJECTS_AMOUNT + difficulty_bonus).min(MAX_OBJECTS_AMOUNT);
    let full_turn_radians = 2.0 * PI;

    for n in 1..=objects_amount {
        let object_kind: ObjectKind = rand::random();
        let object_density: ObjectDensity = rand::random();
        let range = match object_density {
            ObjectDensity::Light => 0.2..=0.4,
            ObjectDensity::Medium => 0.4..=0.6,
            ObjectDensity::Heavy => 0.8..=0.8,
        };
        let distance_from_center_meters: f32 =
            thread_rng().gen_range(range) * LEVEL_BOUNDS_RADIUS_METERS;
        let base_x = distance_from_center_meters * PIXELS_PER_METER;
        let angle = (full_turn_radians / objects_amount as f32) * n as f32;
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

fn spawn_player_and_and_goal(commands: &mut Commands, game_level: &GameLevel) {
    // Flag (goal)
    let flag_extent_x = FLAG_WIDTH_METERS * PIXELS_PER_METER;
    let flag_extent_y = FLAG_HEIGHT_METERS * PIXELS_PER_METER;
    // a point somewhere along the terrain (the inner edge of the level)
    let flag_anchor = game_level
        .elevation_vertices
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
        .insert(Collider::cuboid(flag_extent_x / 2.0, flag_extent_y / 2.0))
        .insert(Restitution::coefficient(1.0));

    // "Player"
    let player_extent_x = PLAYER_WIDTH_METERS * PIXELS_PER_METER;
    let player_extent_y = PLAYER_HEIGHT_METERS * PIXELS_PER_METER;
    let level_bounds_radius_pixels = LEVEL_BOUNDS_RADIUS_METERS * PIXELS_PER_METER;
    let min_player_distance_from_flag = level_bounds_radius_pixels * 1.8;
    let fallback_anchor = &Vec2::new(0.0, level_bounds_radius_pixels * -0.5);
    let player_anchor = game_level
        .elevation_vertices
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

fn game_ui_cleanup(mut commands: Commands, text_query: Query<Entity, With<GameUIText>>) {
    for text in text_query.iter() {
        commands.entity(text).despawn();
    }
}

fn game_ui_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_level: Option<Res<GameLevel>>,
) {
    let font = asset_server.load("VT323-Regular.ttf");
    let font_size = 24.0;

    match game_level {
        None => (),
        Some(level) => {
            commands
                .spawn_bundle(TextBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            bottom: Val::Px(34.0),
                            left: Val::Px(10.0),
                            ..default()
                        },
                        ..default()
                    },
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Level ".to_string(),
                                style: TextStyle {
                                    font: font.clone(),
                                    font_size,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: format!("{}", level.n),
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
                .insert(GameUIText);

            commands
                .spawn_bundle(TextBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            bottom: Val::Px(10.0),
                            left: Val::Px(10.0),
                            ..default()
                        },
                        ..default()
                    },
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Time remaining ".to_string(),
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
                .insert(GameUIText)
                .insert(GameOverCountdownText);
        }
    };
}

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

fn update_game_over_countdown(
    mut game_level: ResMut<GameLevel>,
    mut app_state: ResMut<State<AppState>>,
    time: Res<Time>,
) {
    game_level.countdown_to_game_over.tick(time.delta());

    if game_level.countdown_to_game_over.finished() {
        app_state
            .set(AppState::GameOver)
            .expect("Could not transition to game over state");
    }
}

fn countdown_text_update(
    mut text_query: Query<&mut Text, With<GameOverCountdownText>>,
    game_level: Res<GameLevel>,
) {
    match text_query.get_single_mut() {
        Err(_) => {}
        Ok(mut countdown_text) => {
            let dur_secs = game_level.countdown_to_game_over.duration().as_secs_f64();
            let elapsed = game_level.countdown_to_game_over.elapsed().as_secs_f64();
            let time_remaining = if game_level.countdown_to_game_over.finished() {
                0.0
            } else {
                (dur_secs - elapsed).ceil()
            };

            countdown_text.sections[1].value = format!("{}", time_remaining);
        }
    }
}

fn update_gravity_visuals(
    mut visuals_query: Query<(&mut Path, &mut DrawMode, &mut GravityRing)>,
    gravity_source: Res<GravitySource>,
) {
    let level_bounds_radius_pixels = LEVEL_BOUNDS_RADIUS_METERS * PIXELS_PER_METER;
    let gravity_source_radius_pixels = GRAVITY_SOURCE_RADIUS_METERS * PIXELS_PER_METER;
    let min_radius = gravity_source_radius_pixels;
    // fade to nothing just before the ring hits the terrain
    let max_radius = level_bounds_radius_pixels;

    for (mut path, mut draw_mode, mut ring) in visuals_query.iter_mut() {
        let current_radius = if ring.0 < min_radius {
            max_radius
        } else if ring.0 > max_radius {
            min_radius
        } else {
            ring.0
        };

        // grow or shrink the ring based on gravity force
        let next_radius = current_radius + (gravity_source.force * 0.75);
        // solid when close to the gravity source, transparent when far away
        let opacity = (1.0 - (next_radius / max_radius)).max(0.0);
        // dimmer when transparent
        let lightness = 0.35 + (opacity / 2.0);
        // a little subdued even when close to the gravity source
        let capped_opacity = opacity * 0.8;

        let next_shape = shapes::Circle {
            radius: next_radius,
            center: Vec2::ZERO,
        };
        let next_draw_mode = DrawMode::Stroke(StrokeMode {
            options: StrokeOptions::DEFAULT,
            color: Color::hsla(220.0, 1.0, lightness, capped_opacity),
        });

        ring.0 = next_radius;
        *path = ShapePath::build_as(&next_shape);
        *draw_mode = next_draw_mode;
    }
}
