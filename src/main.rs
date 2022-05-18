use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

enum Attraction {
    Positive,
    Negative,
}

#[derive(Component)]
pub struct GravitySource {
    force: f32,
    cycle: Attraction,
}

#[derive(Component)]
pub struct Attractable {
    move_speed: f32,
    turn_speed: f32,
}

fn main() {
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
        // .add_system(move_attractable)
        // .add_system(rotate_attractable)
        // .add_system(scale_attractable)
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

    // Thing #1
    let hexagon_thing_radius: f32 = 20.0;

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
        .insert(Attractable {
            move_speed: 10.0,
            turn_speed: 0.9,
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(hexagon_thing_radius))
        .insert(Restitution::coefficient(0.2))
        .insert(GravityScale(0.0))
        .insert(ExternalForce {
            force: Vec2::new(0.0, 0.0),
            torque: 0.0,
        });

    // Thing #2
    let cube_thing_extent = 30.0;

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shapes::Rectangle {
                extents: Vec2::new(cube_thing_extent, cube_thing_extent),
                origin: RectangleOrigin::Center,
            },
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::RED)),
            Transform::from_translation(Vec3::new(-100.0, 50.0, 0.0)),
        ))
        .insert(Attractable {
            move_speed: 0.0,
            turn_speed: 0.0,
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(
            cube_thing_extent / 2.0,
            cube_thing_extent / 2.0,
        ))
        .insert(Restitution::coefficient(0.1))
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
        .insert(Attractable {
            move_speed: 0.0,
            turn_speed: 0.0,
        })
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
    if gravity_source.force >= 1.0 {
        gravity_source.cycle = Attraction::Positive;
    } else if gravity_source.force <= -1.0 {
        gravity_source.cycle = Attraction::Negative;
    }

    let force_change = match gravity_source.cycle {
        Attraction::Positive => -timer.delta_seconds(),
        Attraction::Negative => timer.delta_seconds(),
    };

    gravity_source.force += force_change;
}

fn apply_forces(
    mut ext_forces: Query<(&mut ExternalForce, &Transform, &Attractable)>,
    gravity_source: ResMut<GravitySource>,
) {
    for (mut ext_force, transform, _attractable) in ext_forces.iter_mut() {
        let xy: Vec2 = Vec2::new(transform.translation.x, transform.translation.y);

        // TODO this is wrong, distance causes greater force while it should be the other way around
        let dir = xy.normalize() * gravity_source.force * 10.0;
        ext_force.force = dir;
        ext_force.torque = 0.0;
    }
}

// Scratchpad

fn move_attractable(mut attractables: Query<(&mut Transform, &mut Attractable)>, timer: Res<Time>) {
    for (mut transform, attractable) in attractables.iter_mut() {
        let forward = transform.local_x();
        transform.translation += forward * attractable.move_speed * timer.delta_seconds();
    }
}

fn rotate_attractable(
    mut attractables: Query<(&mut Transform, &mut Attractable)>,
    timer: Res<Time>,
) {
    for (mut transform, attractable) in attractables.iter_mut() {
        let incremental_turn_weight = attractable.turn_speed * timer.delta_seconds();
        let old_rotation = transform.rotation;
        let target = old_rotation * Quat::from_rotation_z(0.5);

        transform.rotation = old_rotation.lerp(target, incremental_turn_weight);
    }
}

fn scale_attractable(
    mut attractables: Query<(&mut Transform, &mut Attractable)>,
    timer: Res<Time>,
) {
    let scale_direction = Vec3::new(-1., -1., 0.);
    let scale_speed = 0.25;

    for (mut transform, attractable) in attractables.iter_mut() {
        if transform.scale.x > 0.25 || transform.scale.y > 0.25 {
            transform.scale += scale_direction * scale_speed * timer.delta_seconds();
        }
    }
}
