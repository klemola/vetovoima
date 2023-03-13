use bevy::prelude::*;
use bevy_rapier2d::prelude::ExternalForce;

use crate::app::{ButtonPress, PIXELS_PER_METER};

pub const GRAVITY_SOURCE_RADIUS_METERS: f32 = 2.5;
const GRAVITY_FORCE_SCALE: f32 = 750.0 * GRAVITY_SOURCE_RADIUS_METERS;
const MAX_GRAVITY_FORCE: f32 = 1.0;
const MIN_GRAVITY_FORCE: f32 = -MAX_GRAVITY_FORCE;
const INITIAL_GRAVITY_FORCE: f32 = MAX_GRAVITY_FORCE;
const GRAVITY_AUTO_CYCLE_ENABLED_DEFAULT: bool = false;

#[derive(Component, Resource)]
pub struct GravitySource {
    pub force: f32,
    cycle: Attraction,
    auto_cycle: bool,
}

enum Attraction {
    Positive,
    Negative,
}

impl Default for GravitySource {
    fn default() -> Self {
        Self {
            force: INITIAL_GRAVITY_FORCE,
            cycle: Attraction::Negative,
            auto_cycle: GRAVITY_AUTO_CYCLE_ENABLED_DEFAULT,
        }
    }
}

#[derive(Component)]
pub struct Attractable;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        validate_requirement(
            MAX_GRAVITY_FORCE > MIN_GRAVITY_FORCE,
            "Max gravity should be greater than min gravity",
        );
        validate_requirement(
            GRAVITY_FORCE_SCALE > 0.0,
            "Gravity force scale should be positive",
        );
        validate_requirement(
            INITIAL_GRAVITY_FORCE <= MAX_GRAVITY_FORCE
                && INITIAL_GRAVITY_FORCE >= MIN_GRAVITY_FORCE,
            "Initial gravity should be within gravity force bounds",
        );

        app.insert_resource(GravitySource::default());
    }
}

fn validate_requirement(requirement: bool, description: &str) {
    if !requirement {
        panic!("Validation failed: {description}");
    }
}

pub fn update_gravity(
    mut gravity_source: ResMut<GravitySource>,
    timer: Res<Time>,
    button_press: Res<ButtonPress>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let up_pressed = keyboard_input.pressed(KeyCode::Up) || button_press.up_pressed;
    let down_pressed = keyboard_input.pressed(KeyCode::Down) || button_press.down_pressed;

    let force_change = if gravity_source.auto_cycle {
        let increment = timer.delta_seconds() / 2.0;

        match gravity_source.cycle {
            Attraction::Positive => -increment,
            Attraction::Negative => increment,
        }
    } else {
        let increment = 0.04;

        if up_pressed {
            -increment
        } else if down_pressed {
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

pub fn apply_forces(
    mut ext_forces: Query<(&mut ExternalForce, &Transform), With<Attractable>>,
    gravity_source: ResMut<GravitySource>,
) {
    for (mut ext_force, transform) in ext_forces.iter_mut() {
        let translation_2d: Vec2 = Vec2::new(transform.translation.x, transform.translation.y);

        let force_dir = translation_2d.normalize();
        let base_force = force_dir * gravity_source.force * GRAVITY_FORCE_SCALE;
        let gravity_force = base_force / (translation_2d.length() / PIXELS_PER_METER);
        ext_force.force = gravity_force;
    }
}
