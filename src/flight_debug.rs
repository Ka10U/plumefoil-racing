//! Debug visualizer for physics force vectors, buoyancy probes, and rider dynamics.

use crate::efoil_components::*;
use bevy::prelude::*;

pub struct FlightDebugPlugin;

#[derive(Resource)]
pub struct FlightDebugSettings {
    pub enabled: bool,
    pub force_scale: f32,
}

impl Default for FlightDebugSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            force_scale: 0.003, // 1000 N -> 3 meters arrow
        }
    }
}

impl Plugin for FlightDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlightDebugSettings>()
            .add_systems(Update, (toggle_debug_system, render_flight_gizmos_system));
    }
}

fn toggle_debug_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<FlightDebugSettings>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        settings.enabled = !settings.enabled;
    }
}

fn render_flight_gizmos_system(
    settings: Res<FlightDebugSettings>,
    water: Res<WaterSurfaceResource>,
    mut gizmos: Gizmos,
    query: Query<
        (
            &Transform,
            &HullBuoyancyComponent,
            &HydrofoilWingsComponent,
            &EfoilPropulsionComponent,
            &RiderCounterBalance,
        ),
        With<EfoilRigidBody>,
    >,
) {
    if !settings.enabled {
        return;
    }

    for (transform, buoyancy, wings, propulsion, rider) in &query {
        let com = transform.translation;
        let scale = settings.force_scale;

        // 1. Center of Mass (white sphere)
        gizmos.sphere(com, 0.06, Color::WHITE);

        // 2. Buoyancy Probes (blue when submerged, cyan when floating/dry)
        for probe in &buoyancy.config.probes {
            let probe_world = transform.transform_point(probe.local_position);
            let water_y = (water.wave_height_fn)(probe_world.x, probe_world.z);
            let is_submerged = probe_world.y < water_y;

            let color = if is_submerged {
                Color::srgb(0.1, 0.5, 0.95)
            } else {
                Color::srgb(0.3, 0.9, 0.4)
            };

            gizmos.sphere(probe_world, 0.04, color);
        }

        // 3. Front Wing Lift & Drag
        let front_world = transform.transform_point(wings.front_wing_local_pos);
        if wings.last_lift_force.length_squared() > 1.0 {
            let lift_end = front_world + wings.last_lift_force * scale;
            gizmos.arrow(front_world, lift_end, Color::srgb(0.1, 0.95, 0.3)); // Bright green
        }
        if wings.last_drag_force.length_squared() > 1.0 {
            let drag_end = front_world + wings.last_drag_force * scale;
            gizmos.arrow(front_world, drag_end, Color::srgb(0.95, 0.2, 0.2)); // Red
        }

        // 4. Motor Thrust
        let motor_world = transform.transform_point(propulsion.motor_local_pos);
        if propulsion.last_thrust_force.length_squared() > 1.0 {
            let thrust_end = motor_world + propulsion.last_thrust_force * scale;
            gizmos.arrow(motor_world, thrust_end, Color::srgb(0.95, 0.85, 0.1)); // Yellow
        }

        // 5. Rider Counter-Balancing Force Vector (Gravity + Centrifugal)
        let stance_world = transform.transform_point(
            rider.nominal_stance_pos
                + Vec3::new(
                    rider.roll_lean * rider.max_roll_arm,
                    0.0,
                    rider.pitch_lean * rider.max_pitch_arm,
                ),
        );
        gizmos.sphere(stance_world, 0.05, Color::srgb(0.8, 0.2, 0.9)); // Purple stance sphere

        if rider.last_rider_force.length_squared() > 1.0 {
            let rider_end = stance_world + rider.last_rider_force * scale;
            gizmos.arrow(stance_world, rider_end, Color::srgb(0.85, 0.3, 0.95)); // Purple arrow
        }
    }
}
