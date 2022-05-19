use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct GravitySource {
    force: f32,
    cycle: Attraction,
}

enum Attraction {
    Positive,
    Negative,
}

#[derive(Component)]
pub struct Attractable {}

#[derive(Component)]
pub struct Player {
    turn_speed: f32,
    move_speed: f32,
}

const GRAVITY_FORCE_SCALE: f32 = 1600.0;
const MAX_GRAVITY_FORCE: f32 = 1.0;
const MIN_GRAVITY_FORCE: f32 = -MAX_GRAVITY_FORCE;

fn main() {
    assert!(MAX_GRAVITY_FORCE > MIN_GRAVITY_FORCE);
    assert!(GRAVITY_FORCE_SCALE > 0.0);

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GravitySource {
            force: 0.0,
            cycle: Attraction::Negative,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_system(update_gravity)
        .add_system(apply_forces)
        .add_system(update_player_velocity)
        // .add_system(move_player)
        // .add_system(rotate_player)
        // .add_system(scale_player)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // World (the hollow circle that bounds the simulation)
    let world_radius = 400.0;
    let world_thickness = 1.0;
    let world_shape = shapes::Circle {
        radius: world_radius,
        center: Vec2::ZERO,
    };
    let world_vertices: Vec<Vec2> = (0..=360)
        .map(|a: i32| {
            let a_rad: f32 = a as f32 * (PI / 180.0);
            let r = world_radius - world_thickness;
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
    let gravity_source_radius: f32 = 50.0;
    let gravity_source_shape = shapes::Circle {
        radius: gravity_source_radius,
        center: Vec2::ZERO,
    };

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &gravity_source_shape,
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::WHITE)),
            Transform::default(),
        ))
        .insert(RigidBody::Fixed)
        .insert(Collider::ball(gravity_source_radius))
        .insert(Restitution::coefficient(0.1));

    // "Player"
    let player_extent_x = 10.0;
    let player_extent_y = 40.0;

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
        .insert(Player {
            move_speed: 0.0,
            turn_speed: 0.0,
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(
            player_extent_x / 2.0,
            player_extent_y / 2.0,
        ))
        .insert(MassProperties {
            local_center_of_mass: Vec2::new(0.0, -player_extent_y),
            mass: 1000.0,
            principal_inertia: 0.2,
        })
        .insert(Restitution::coefficient(0.1))
        .insert(GravityScale(0.0))
        .insert(ExternalForce {
            force: Vec2::new(0.0, 0.0),
            torque: 0.0,
        })
        .insert(Velocity {
            linvel: Vec2::ZERO,
            angvel: 0.0,
        });

    // Thing #1
    let hexagon_thing_radius: f32 = 22.0;
    let step_angle = PI / 3.0;
    let hexagon_vertices = vec![
        Vec2::new(hexagon_thing_radius, 0.0),
        Vec2::new(
            step_angle.cos() * hexagon_thing_radius,
            step_angle.sin() * hexagon_thing_radius,
        ),
        Vec2::new(
            (step_angle * 2.0).cos() * hexagon_thing_radius,
            (step_angle * 2.0).sin() * hexagon_thing_radius,
        ),
        Vec2::new(-hexagon_thing_radius, 0.0),
        Vec2::new(
            -(step_angle * 2.0).cos() * hexagon_thing_radius,
            -(step_angle * 2.0).sin() * hexagon_thing_radius,
        ),
        Vec2::new(
            -step_angle.cos() * hexagon_thing_radius,
            -step_angle.sin() * hexagon_thing_radius,
        ),
    ];

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shapes::RegularPolygon {
                sides: 6,
                feature: shapes::RegularPolygonFeature::Radius(hexagon_thing_radius),
                ..shapes::RegularPolygon::default()
            },
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::RED)),
            Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        ))
        .insert(Attractable {})
        .insert(RigidBody::Dynamic)
        .insert(
            Collider::convex_hull(&hexagon_vertices)
                .unwrap_or(Collider::ball(hexagon_thing_radius)),
        )
        .insert(Restitution::coefficient(0.1))
        .insert(GravityScale(0.0))
        .insert(ExternalForce {
            force: Vec2::new(0.0, 0.0),
            torque: 0.0,
        });

    // Thing #2
    let cube_thing_extent = 22.0;

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shapes::Rectangle {
                extents: Vec2::new(cube_thing_extent, cube_thing_extent),
                origin: RectangleOrigin::Center,
            },
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::RED)),
            Transform::from_translation(Vec3::new(-100.0, 50.0, 0.0)),
        ))
        .insert(Attractable {})
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(
            cube_thing_extent / 2.0,
            cube_thing_extent / 2.0,
        ))
        .insert(Restitution::coefficient(0.3))
        .insert(GravityScale(0.0))
        .insert(ExternalForce {
            force: Vec2::new(0.0, 0.0),
            torque: 0.0,
        });

    // Thing #3
    let small_thing_radius: f32 = 10.0;
    let small_thing_shape = shapes::Circle {
        radius: small_thing_radius,
        center: Vec2::ZERO,
    };

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &small_thing_shape,
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::RED)),
            Transform::from_translation(Vec3::new(-10.0, -50.0, 0.0)),
        ))
        .insert(Attractable {})
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(small_thing_radius))
        .insert(Restitution::coefficient(1.0))
        .insert(GravityScale(0.0))
        .insert(ExternalForce {
            force: Vec2::new(0.0, 0.0),
            torque: 0.0,
        });
}

fn update_gravity(mut gravity_source: ResMut<GravitySource>, timer: Res<Time>) {
    if gravity_source.force >= MAX_GRAVITY_FORCE {
        // Enforce force upper limit
        gravity_source.force = MAX_GRAVITY_FORCE;
        gravity_source.cycle = Attraction::Positive;
    } else if gravity_source.force <= MIN_GRAVITY_FORCE {
        // Enforce force lower limit
        gravity_source.force = MIN_GRAVITY_FORCE;
        gravity_source.cycle = Attraction::Negative;
    }

    let increment = timer.delta_seconds() / 2.0;
    let force_change = match gravity_source.cycle {
        Attraction::Positive => -increment,
        Attraction::Negative => increment,
    };

    gravity_source.force += force_change;
}

fn update_player_velocity(mut velocities: Query<(&mut Velocity, &Transform), With<Player>>) {
    let (mut vel, transform) = velocities.single_mut();
    let forward = transform.local_x();
    let player_control_dir: Vec2 = Vec2::new(forward.x, forward.y).normalize();
    let player_control_force = player_control_dir * 0.5;

    vel.linvel += player_control_force;
    vel.angvel = 0.0;
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
        ext_force.torque = 0.0;
    }
}

// Scratchpad

fn move_player(mut player_query: Query<(&mut Transform, &mut Player)>, timer: Res<Time>) {
    let (mut transform, player) = player_query.single_mut();
    let forward = transform.local_x();

    transform.translation += forward * player.move_speed * timer.delta_seconds();
}

fn rotate_player(mut player_query: Query<(&mut Transform, &mut Player)>, timer: Res<Time>) {
    let (mut transform, player) = player_query.single_mut();
    let incremental_turn_weight = player.turn_speed * timer.delta_seconds();
    let old_rotation = transform.rotation;
    let target = old_rotation * Quat::from_rotation_z(0.5);

    transform.rotation = old_rotation.lerp(target, incremental_turn_weight);
}

fn scale_player(mut transform_query: Query<&mut Transform, With<Player>>, timer: Res<Time>) {
    let mut transform = transform_query.single_mut();
    let scale_direction = Vec3::new(-1., -1., 0.);
    let scale_speed = 0.25;

    if transform.scale.x > 0.25 || transform.scale.y > 0.25 {
        transform.scale += scale_direction * scale_speed * timer.delta_seconds();
    }
}
