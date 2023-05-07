use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::plugin::RapierContext;
use bevy_rapier2d::prelude::*;
use rand::{prelude::*, seq::IteratorRandom, Rng};
use rand_distr::{Distribution, Normal, Standard};
use std::f32::consts::PI;
use std::time::Duration;

use crate::app::{cursor_visible, AppState, ButtonPress, VetovoimaColor, PIXELS_PER_METER};
use crate::simulation::{
    apply_forces, update_gravity, Attractable, GravitySource, GRAVITY_SOURCE_RADIUS_METERS,
};

const LEVEL_BOUNDS_RADIUS_METERS: f32 = 28.0;
const PLAYER_WIDTH_METERS: f32 = 0.8;
const PLAYER_HEIGHT_METERS: f32 = 1.8;
const FLAG_WIDTH_METERS: f32 = 0.55;
const FLAG_HEIGHT_METERS: f32 = LEVEL_BOUNDS_RADIUS_METERS / 4.8;

const PLAYER_MAX_FORWARD_VELOCITY: f32 = 64.0;
const PLAYER_SLOW_DOWN_VELOCITY: f32 = -200.0;
const PLAYER_MAX_ANGULAR_VELOCITY: f32 = 90.0;

const BASE_OBJECTS_AMOUNT: u32 = 16;
const MAX_OBJECTS_AMOUNT: u32 = 60;

const LOADING_TIMER_DURATION_SECONDS: f32 = 3.0;

const DEFAULT_COLLISION_GROUP: CollisionGroups =
    CollisionGroups::new(Group::GROUP_1, Group::GROUP_1);
const SECONDARY_COLLISION_GROUP: CollisionGroups =
    CollisionGroups::new(Group::GROUP_2, Group::GROUP_2);

pub enum GameEvent {
    CountdownTick(u32),
    GoalReached,
    GameOver,
    LevelStarted,
    PlayerCollided(Duration, f32),
}

#[derive(Component, Clone, Debug, Resource)]
pub struct GameLevel {
    n: u32,
    countdown_to_game_over: Timer,
    terrain_vertices: Vec<Vec2>,
    elevation_vertices: Vec<Vec2>,
}

#[derive(Resource)]
struct PlayerCollision {
    previous_collision_time: Duration,
}

impl Default for PlayerCollision {
    fn default() -> Self {
        PlayerCollision {
            previous_collision_time: Duration::from_millis(0),
        }
    }
}

enum ObjectKind {
    Ngon,
    Circle,
}

impl Distribution<ObjectKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ObjectKind {
        let distance: f32 = rng.gen();

        if distance > 0.6 {
            ObjectKind::Circle
        } else {
            ObjectKind::Ngon
        }
    }
}

#[derive(PartialEq)]
enum ObjectDensity {
    Light,
    Medium,
    Heavy,
}

impl Distribution<ObjectDensity> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ObjectDensity {
        let distance: f32 = rng.gen();

        if distance > 0.78 {
            ObjectDensity::Heavy
        } else if distance < 0.3 {
            ObjectDensity::Light
        } else {
            ObjectDensity::Medium
        }
    }
}

#[derive(Component, Resource)]
struct LoadingState(Timer);

#[derive(Component)]
struct LoadingScreen;

#[derive(Component)]
struct LoadingLevelText;

#[derive(Component)]
struct GameObject;

#[derive(Component)]
struct GravityRing(f32);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct Flag;

#[derive(Component)]
struct FlagAura(f32);

#[derive(Component)]
struct GameUI;

#[derive(Component)]
struct GameOverCountdownText;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .add_event::<GameEvent>()
            .insert_resource(PlayerCollision::default())
            .add_systems(
                (cursor_visible::<false>, loading_screen_setup, game_setup)
                    .in_schedule(OnEnter(AppState::LoadingLevel)),
            )
            .add_system(loading_update.in_set(OnUpdate(AppState::LoadingLevel)))
            .add_system(loading_finished_effects.in_schedule(OnExit(AppState::LoadingLevel)))
            .add_system(game_ui_setup.in_schedule(OnEnter(AppState::InGame)))
            .add_systems(
                (
                    update_gravity,
                    apply_forces,
                    update_player_velocity,
                    update_flag_aura,
                    check_goal_reached,
                    detect_player_collision,
                    update_game_over_countdown,
                    countdown_text_update,
                    update_gravity_visuals,
                )
                    .in_set(OnUpdate(AppState::InGame)),
            )
            .add_systems((game_cleanup, game_ui_cleanup).in_schedule(OnExit(AppState::InGame)));
    }
}

fn create_game_level(current_level_value: u32) -> GameLevel {
    let next_level_n = current_level_value + 1;
    let radius_pixels = LEVEL_BOUNDS_RADIUS_METERS * PIXELS_PER_METER;
    // the outer edge (rim) of the circle polygon
    let outer_circle_steps = 180;
    let rim_vertices: Vec<Vec2> = (0..=outer_circle_steps)
        .map(|step: u32| {
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
                let std_deviation = 0.086 * PIXELS_PER_METER;
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

    let countdown: u64 = match next_level_n {
        1..=4 => 60,
        5..=9 => 40,
        10..=25 => 30,
        _ => 20,
    };

    GameLevel {
        n: next_level_n,
        countdown_to_game_over: Timer::new(Duration::from_secs(countdown), TimerMode::Once),
        terrain_vertices: vec![elevation_vertices.clone(), rim_vertices].concat(),
        elevation_vertices,
    }
}

fn game_cleanup(mut commands: Commands, game_object_query: Query<Entity, With<GameObject>>) {
    for object in game_object_query.iter() {
        commands.entity(object).despawn();
    }
}

fn game_setup(
    mut commands: Commands,
    game_level: Option<Res<GameLevel>>,
    mut gravity_source: ResMut<GravitySource>,
    mut player_collision: ResMut<PlayerCollision>,
) {
    let current_game_level_n = match game_level {
        Some(level) => level.n,
        None => 0,
    };
    let next_game_level = create_game_level(current_game_level_n);

    // Reset some resources
    *gravity_source = GravitySource::default();
    *player_collision = PlayerCollision::default();

    // Will replace the current game level with the next
    commands.insert_resource(next_game_level.clone());

    spawn_level(&mut commands, &next_game_level);
    spawn_objects(&mut commands, next_game_level.n);
    spawn_player_and_and_goal(&mut commands, &next_game_level);
}

fn loading_screen_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let loading_timer = Timer::from_seconds(LOADING_TIMER_DURATION_SECONDS, TimerMode::Once);
    let font = asset_server.load("VT323-Regular.ttf");
    let font_size = 100.0;

    commands.insert_resource(LoadingState(loading_timer));

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
            container
                .spawn(TextBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Level ".to_string(),
                                style: TextStyle {
                                    font: font.clone(),
                                    font_size,
                                    color: VetovoimaColor::WHITEISH,
                                },
                            },
                            TextSection {
                                value: "".to_string(),
                                style: TextStyle {
                                    font,
                                    font_size,
                                    color: VetovoimaColor::YELLOWISH,
                                },
                            },
                        ],
                        ..default()
                    },
                    ..default()
                })
                .insert(LoadingLevelText);
        })
        .insert(LoadingScreen);
}

fn spawn_level(commands: &mut Commands, game_level: &GameLevel) {
    let level_shape = &shapes::Polygon {
        points: game_level.terrain_vertices.clone(),
        closed: true,
    };

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(level_shape),
            ..Default::default()
        },
        GameObject,
        Fill {
            options: FillOptions::default(),
            color: VetovoimaColor::WHITEISH,
        },
        Collider::polyline(game_level.elevation_vertices.clone(), None),
        DEFAULT_COLLISION_GROUP,
    ));

    // Gravity source (as a visual/physics object)
    let gravity_source_radius_pixels = GRAVITY_SOURCE_RADIUS_METERS * PIXELS_PER_METER;
    let gravity_source_shape = shapes::Circle {
        radius: gravity_source_radius_pixels,
        center: Vec2::ZERO,
    };

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&gravity_source_shape),
            ..Default::default()
        },
        GameObject,
        Fill {
            options: FillOptions::DEFAULT,
            color: VetovoimaColor::WHITEISH,
        },
        RigidBody::Fixed,
        Collider::ball(gravity_source_radius_pixels),
        Restitution::coefficient(0.1),
        DEFAULT_COLLISION_GROUP,
    ));

    // Gravity force visualization
    let gravity_rings_amount = 6;
    let level_bounds_radius_pixels = LEVEL_BOUNDS_RADIUS_METERS * PIXELS_PER_METER;
    let ring_frequency =
        (level_bounds_radius_pixels - gravity_source_radius_pixels) / gravity_rings_amount as f32;

    for n in 1..=gravity_rings_amount {
        let n_f = n as f32;
        let radius = ring_frequency * n_f;
        let shape = shapes::Circle {
            radius,
            center: Vec2::ZERO,
        };

        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..Default::default()
            },
            Stroke::new(Color::hsla(0.0, 1.0, 1.0, 0.0), 1.0),
            GravityRing(radius),
            GameObject,
        ));
    }
}

fn spawn_objects(commands: &mut Commands, game_level_n: u32) {
    let difficulty_bonus = 2 * game_level_n;
    let objects_amount = (BASE_OBJECTS_AMOUNT + difficulty_bonus).min(MAX_OBJECTS_AMOUNT);
    let full_turn_radians = 2.0 * PI;

    for n in 1..=objects_amount {
        let object_density: ObjectDensity = rand::random();
        let (object_kind, distance_range) = match object_density {
            ObjectDensity::Light => (ObjectKind::Circle, 0.15..=0.85),
            ObjectDensity::Medium => (rand::random(), 0.25..=0.75),
            ObjectDensity::Heavy => (ObjectKind::Ngon, 0.5..=0.85),
        };
        let distance_from_center_meters: f32 =
            thread_rng().gen_range(distance_range) * LEVEL_BOUNDS_RADIUS_METERS;
        let base_x = distance_from_center_meters * PIXELS_PER_METER;
        let angle_radians = (full_turn_radians / objects_amount as f32) * n as f32;
        let mut transform = Transform::from_translation(Vec3::new(base_x, 0.0, 0.0));

        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_z(angle_radians));
        spawn_object(commands, object_kind, object_density, transform);
    }
}

fn spawn_object(
    commands: &mut Commands,
    kind: ObjectKind,
    density: ObjectDensity,
    transform: Transform,
) {
    let (density_value, base_scale_factor, color, max_random_force, max_random_torque) =
        match density {
            ObjectDensity::Light => (0.75, 1.0, VetovoimaColor::GREENISH, 3.0, 0.3),
            ObjectDensity::Medium => (1.0, 2.0, VetovoimaColor::REDDISH, 30.0, 1.0),
            ObjectDensity::Heavy => (10.0, 3.2, VetovoimaColor::WHITEISH, 200.0, 20.0),
        };
    let scale_variation: f32 = thread_rng().gen_range(-0.2..0.4);
    let scale_factor = (base_scale_factor + (base_scale_factor * scale_variation)).max(1.0);
    let (path, collider, restitution_coefficient) = match kind {
        ObjectKind::Ngon => ngon_props(scale_factor),
        ObjectKind::Circle => circle_props(scale_factor),
    };

    commands.spawn((
        ShapeBundle {
            path,
            transform,
            ..Default::default()
        },
        Fill {
            options: FillOptions::default(),
            color: color,
        },
        GameObject,
        Attractable,
        RigidBody::Dynamic,
        collider,
        ColliderMassProperties::Density(density_value),
        Restitution::coefficient(restitution_coefficient),
        GravityScale(0.0),
        random_external_force(max_random_force, max_random_torque),
        CollisionGroups::new(
            DEFAULT_COLLISION_GROUP
                .memberships
                .union(SECONDARY_COLLISION_GROUP.memberships),
            DEFAULT_COLLISION_GROUP
                .filters
                .union(SECONDARY_COLLISION_GROUP.filters),
        ),
    ));
}

fn ngon_props(scale_factor: f32) -> (Path, Collider, f32) {
    let base_radius: f32 = 0.5 * scale_factor * PIXELS_PER_METER;
    let full_turn_radians = 2.0 * PI;
    let std_deviation = 1.3;
    let normal_distribution = Normal::new(base_radius, std_deviation).unwrap();
    let sides_amount: u32 = thread_rng().gen_range(5..=15);
    let ngon_vertices: Vec<Vec2> = (1..=sides_amount)
        .into_iter()
        .map(|side_n| {
            let angle_radians = (full_turn_radians / sides_amount as f32) * side_n as f32;
            let distance = normal_distribution.sample(&mut rand::thread_rng());

            // polar -> cartesian conversion
            Vec2::new(
                distance * angle_radians.cos(),
                distance * angle_radians.sin(),
            )
        })
        .collect();

    let shape = shapes::Polygon {
        points: ngon_vertices.clone(),
        closed: true,
    };
    let path = GeometryBuilder::build_as(&shape);
    let collider =
        Collider::convex_hull(&ngon_vertices).unwrap_or_else(|| Collider::ball(base_radius));

    (path, collider, 0.1)
}

fn circle_props(scale_factor: f32) -> (Path, Collider, f32) {
    let radius: f32 = 0.5 * scale_factor * PIXELS_PER_METER;

    let shape = shapes::Circle {
        radius,
        center: Vec2::ZERO,
    };
    let path = GeometryBuilder::build_as(&shape);
    let collider = Collider::ball(radius);

    (path, collider, 1.0)
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
    let flag_transform = stand_upright_at_anchor(flag_anchor, flag_extent_y);

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shapes::Rectangle {
                extents: Vec2::new(flag_extent_x, flag_extent_y),
                origin: RectangleOrigin::Center,
            }),
            transform: flag_transform,
            ..Default::default()
        },
        GameObject,
        Fill {
            options: FillOptions::default(),
            color: VetovoimaColor::BLUEISH_LIGHT,
        },
        Flag,
        RigidBody::Fixed,
        Collider::cuboid(flag_extent_x / 1.8, flag_extent_y / 2.0),
        Restitution::coefficient(1.0),
        DEFAULT_COLLISION_GROUP,
    ));

    // Flag force aura (sweeps gravity objects away from the flag to let the player in)
    let aura_shape = shapes::Circle {
        radius: 0.0,
        center: Vec2::ZERO,
    };

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&aura_shape),
            transform: Transform::from_translation(flag_transform.translation),
            ..Default::default()
        },
        Stroke::new(Color::hsla(0.0, 1.0, 1.0, 0.0), 1.0),
        GameObject,
        FlagAura(0.0),
        RigidBody::Fixed,
        Collider::ball(1.0),
        Restitution::coefficient(1.0),
        SECONDARY_COLLISION_GROUP,
    ));

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
    let player_transform = stand_upright_at_anchor(player_anchor, player_extent_y);

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shapes::Rectangle {
                extents: Vec2::new(player_extent_x, player_extent_y),
                origin: RectangleOrigin::Center,
            }),
            transform: player_transform,
            ..Default::default()
        },
        GameObject,
        Fill {
            options: FillOptions::default(),
            color: VetovoimaColor::YELLOWISH,
        },
        Attractable,
        Player,
        RigidBody::Dynamic,
        Collider::cuboid(player_extent_x / 2.0, player_extent_y / 2.0),
        ActiveEvents::CONTACT_FORCE_EVENTS,
        ContactForceEventThreshold(225.0),
        AdditionalMassProperties::MassProperties(MassProperties {
            local_center_of_mass: Vec2::new(0.0, -player_extent_y),
            mass: 0.3,
            principal_inertia: 0.0,
        }),
        Restitution::coefficient(0.1),
        GravityScale(0.0),
        ExternalForce {
            force: Vec2::ZERO,
            torque: 0.0,
        },
        Velocity {
            linvel: Vec2::ZERO,
            angvel: 0.0,
        },
        DEFAULT_COLLISION_GROUP,
    ));
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

fn game_ui_cleanup(mut commands: Commands, ui_query: Query<Entity, With<GameUI>>) {
    for ui_entity in ui_query.iter() {
        commands.entity(ui_entity).despawn_recursive();
    }
}

fn game_ui_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("VT323-Regular.ttf");
    let font_size = 36.0;

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(GameUI)
        .with_children(|container| {
            container
                .spawn(TextBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font,
                            font_size,
                            color: VetovoimaColor::BLACKISH,
                        },
                    ),

                    ..default()
                })
                .insert(GameOverCountdownText);
        });
}

fn loading_update(
    mut commands: Commands,
    mut level_text_query: Query<&mut Text, With<LoadingLevelText>>,
    mut loading: ResMut<LoadingState>,
    mut app_state: ResMut<NextState<AppState>>,
    game_level: Option<Res<GameLevel>>,
    time: Res<Time>,
) {
    loading.0.tick(time.delta());

    match game_level {
        Some(level) => {
            let mut text = level_text_query
                .get_single_mut()
                .expect("Level text doesn't exist during loading!");
            text.sections[1].value = format!("{}", level.n)
        }
        None => (),
    };

    if loading.0.finished() {
        commands.remove_resource::<LoadingState>();
        app_state.set(AppState::InGame);
    }
}

fn loading_finished_effects(
    mut commands: Commands,
    loading_screen_query: Query<Entity, With<LoadingScreen>>,
    mut game_event: EventWriter<GameEvent>,
) {
    let loading_screen = loading_screen_query
        .get_single()
        .expect("Loading screen doesn't exist while loading a level!");

    commands.entity(loading_screen).despawn_recursive();
    game_event.send(GameEvent::LevelStarted);
}

fn update_player_velocity(
    mut velocities: Query<(&mut Velocity, &Transform), With<Player>>,
    button_press: Res<ButtonPress>,
    time: Res<Time>,
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
            let intensity = if button_press.left_pressed && negative_velocity == 0.0 {
                // Slow down until the player halts
                PLAYER_SLOW_DOWN_VELOCITY
            } else if button_press.right_pressed && current_velocity < PLAYER_MAX_FORWARD_VELOCITY {
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

            vel.linvel += player_control_force * time.delta_seconds();
            vel.angvel = (angular_velocity * time.delta_seconds())
                .clamp(-PLAYER_MAX_ANGULAR_VELOCITY, PLAYER_MAX_ANGULAR_VELOCITY);
        }
    }
}

fn update_flag_aura(mut aura_query: Query<(&mut Path, &mut Stroke, &mut Collider, &mut FlagAura)>) {
    let min_radius = 0.0;
    let max_radius = 100.0;
    let hue = 35.0;
    let radius_delta = 0.75;

    match aura_query.get_single_mut() {
        Err(_) => {
            // Aura should always exist at this phase
        }
        Ok((mut path, mut stroke, mut collider, mut aura)) => {
            let (next_radius, next_color) =
                update_ring(aura.0, radius_delta, min_radius, max_radius, hue);

            let next_shape = shapes::Circle {
                radius: next_radius,
                center: Vec2::ZERO,
            };

            aura.0 = next_radius;
            stroke.color = next_color;
            *path = ShapePath::build_as(&next_shape);

            let next_collider = Collider::ball(next_radius);
            *collider = next_collider;
        }
    }
}

fn check_goal_reached(
    player_query: Query<(&Transform, &Collider), With<Player>>,
    flag_query: Query<Entity, With<Flag>>,
    mut game_event: EventWriter<GameEvent>,
    mut app_state: ResMut<NextState<AppState>>,
    rapier_context: Res<RapierContext>,
) {
    if let (Ok((player_transform, player_shape)), Ok(flag_entity)) =
        (player_query.get_single(), flag_query.get_single())
    {
        let shape_pos: Vec2 = Vec2::new(
            player_transform.translation.x,
            player_transform.translation.y,
        );
        let (_, player_angle) = player_transform.rotation.to_axis_angle();
        let flag_id = flag_entity.index();

        rapier_context.intersections_with_shape(
            shape_pos,
            player_angle,
            player_shape,
            QueryFilter::predicate(QueryFilter::only_fixed(), &|entity: Entity| {
                entity.index() == flag_id
            }),
            |_| {
                game_event.send(GameEvent::GoalReached);
                app_state.set(AppState::LoadingLevel);

                true
            },
        );
    }
}

fn detect_player_collision(
    mut contact_force_events: EventReader<ContactForceEvent>,
    mut game_event: EventWriter<GameEvent>,
    mut player_collision: ResMut<PlayerCollision>,
    time: Res<Time>,
) {
    for event in contact_force_events.iter() {
        let elapsed = time.elapsed();
        let time_since_previous_collision = elapsed - player_collision.previous_collision_time;

        player_collision.previous_collision_time = elapsed;

        if time_since_previous_collision > Duration::from_millis(50) {
            game_event.send(GameEvent::PlayerCollided(
                time_since_previous_collision,
                event.total_force_magnitude,
            ));
        }
    }
}

fn update_game_over_countdown(
    mut game_level: ResMut<GameLevel>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_event: EventWriter<GameEvent>,
    time: Res<Time>,
) {
    let time_remaining = timer_to_secs_remaining(&game_level.countdown_to_game_over);

    game_level.countdown_to_game_over.tick(time.delta());

    let next_time_remaining = timer_to_secs_remaining(&game_level.countdown_to_game_over);

    if game_level.countdown_to_game_over.finished() {
        game_event.send(GameEvent::GameOver);
        app_state.set(AppState::GameOver);
    } else if next_time_remaining != time_remaining {
        game_event.send(GameEvent::CountdownTick(next_time_remaining));
    }
}

fn countdown_text_update(
    mut text_query: Query<&mut Text, With<GameOverCountdownText>>,
    game_level: Res<GameLevel>,
) {
    match text_query.get_single_mut() {
        Err(_) => {}
        Ok(mut countdown_text) => {
            let seconds_remaining = timer_to_secs_remaining(&game_level.countdown_to_game_over);
            let (text_color, font_size) = if seconds_remaining <= 5 {
                (VetovoimaColor::REDDISH, 48.0)
            } else {
                (VetovoimaColor::BLACKISH, 36.0)
            };

            let countdown_text_seconds = &mut countdown_text.sections[0];

            countdown_text_seconds.style.color = text_color;
            countdown_text_seconds.style.font_size = font_size;
            countdown_text_seconds.value = format!("{}", seconds_remaining);
        }
    }
}

fn update_gravity_visuals(
    mut visuals_query: Query<(&mut Path, &mut Stroke, &mut GravityRing)>,
    gravity_source: Res<GravitySource>,
) {
    let level_bounds_radius_pixels = LEVEL_BOUNDS_RADIUS_METERS * PIXELS_PER_METER;
    let gravity_source_radius_pixels = GRAVITY_SOURCE_RADIUS_METERS * PIXELS_PER_METER;
    let min_radius = gravity_source_radius_pixels;
    // fade to nothing just before the ring hits the terrain
    let max_radius = level_bounds_radius_pixels;
    let radius_force_ratio = PIXELS_PER_METER / 21.35;
    let radius_delta = gravity_source.force * radius_force_ratio;
    let hue = 220.0;

    for (mut path, mut stroke, mut ring) in visuals_query.iter_mut() {
        let (next_radius, next_color) =
            update_ring(ring.0, radius_delta, min_radius, max_radius, hue);
        let next_shape = shapes::Circle {
            radius: next_radius,
            center: Vec2::ZERO,
        };

        ring.0 = next_radius;
        stroke.color = next_color;
        *path = ShapePath::build_as(&next_shape);
    }
}

// Helper functions

fn timer_to_secs_remaining(timer: &Timer) -> u32 {
    let dur_secs = timer.duration().as_secs_f64();
    let elapsed = timer.elapsed().as_secs_f64();

    if timer.finished() {
        0
    } else {
        (dur_secs - elapsed).ceil() as u32
    }
}

fn update_ring(
    current_radius: f32,
    radius_delta: f32,
    min_radius: f32,
    max_radius: f32,
    hue: f32,
) -> (f32, Color) {
    let current_radius = if current_radius < min_radius {
        max_radius
    } else if current_radius > max_radius {
        min_radius
    } else {
        current_radius
    };

    let next_radius = current_radius + radius_delta;
    // solid when close to the origin, transparent when far away
    let opacity_value: f32 = 1.0 - (next_radius / max_radius);
    let opacity = opacity_value.max(0.0);
    // dimmer when transparent
    let lightness = 0.35 + (opacity / 2.0);
    // a little subdued even when close to the origin
    let capped_opacity = opacity * 0.8;

    let next_color = Color::hsla(hue, 1.0, lightness, capped_opacity);

    (next_radius, next_color)
}

fn random_external_force(max_force: f32, max_torque: f32) -> ExternalForce {
    let mut rng = thread_rng();
    let force_abs = max_force.abs();

    let x = rng.gen_range(-force_abs..force_abs);
    let y = rng.gen_range(-force_abs..force_abs);
    let force: Vec2 = Vec2::new(x, y);
    let torque = rng.gen_range(0.0..max_torque.abs());

    ExternalForce { force, torque }
}
