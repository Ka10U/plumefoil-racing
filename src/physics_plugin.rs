//! Physics plugin integrating Avian3D with deterministic hydrodynamics,
//! wing lift/drag, Archimedes buoyancy, rider counter-balance, and propulsion.

#![allow(clippy::type_complexity)]

use avian3d::prelude::*;
use bevy::prelude::*;
use plumefoil_core::constants::{SEAWATER_DENSITY, STANDARD_GRAVITY, mps_to_kmh, mps_to_knots};
use plumefoil_physics::buoyancy::calculate_hull_buoyancy;
use plumefoil_physics::lift_drag::calculate_wing_forces;
use plumefoil_physics::thrust::calculate_motor_thrust;

use crate::efoil_components::*;

pub struct EfoilPhysicsPlugin;

impl Plugin for EfoilPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterSurfaceResource>()
            .add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vec3::new(0.0, -STANDARD_GRAVITY, 0.0)))
            .add_systems(
                FixedUpdate,
                (
                    clear_external_forces_system,
                    accumulate_propulsion_system,
                    accumulate_buoyancy_system,
                    accumulate_hydrofoil_system,
                    accumulate_rider_dynamics_system,
                    update_telemetry_system,
                )
                    .chain(),
            );
    }
}

/// Resets external forces and torques at the beginning of each fixed physics step.
fn clear_external_forces_system(
    mut query: Query<(&mut ExternalForce, &mut ExternalTorque), With<EfoilRigidBody>>,
) {
    for (mut ext_force, mut ext_torque) in &mut query {
        ext_force.clear();
        ext_torque.clear();
    }
}

/// Evaluates electric motor propeller propulsion.
fn accumulate_propulsion_system(
    time: Res<Time>,
    mut query: Query<
        (
            &Transform,
            &LinearVelocity,
            &mut EfoilPropulsionComponent,
            &mut ExternalForce,
            &mut ExternalTorque,
        ),
        With<EfoilRigidBody>,
    >,
) {
    let dt = time.delta_secs();

    for (transform, linear_vel, mut propulsion, mut ext_force, mut ext_torque) in &mut query {
        // Smooth throttle response
        let rate = dt / propulsion.config.response_time_constant.max(0.01);
        propulsion.current_throttle +=
            (propulsion.target_throttle - propulsion.current_throttle) * rate.clamp(0.0, 1.0);

        let forward = transform.forward().as_vec3();
        let advance_speed = linear_vel.0.dot(forward).max(0.0);

        let thrust = calculate_motor_thrust(
            &propulsion.config,
            propulsion.current_throttle,
            forward,
            advance_speed,
        );

        propulsion.last_thrust_force = thrust;

        // Apply thrust force at motor hub position
        let motor_world = transform.transform_point(propulsion.motor_local_pos);
        let lever_arm = motor_world - transform.translation;
        let torque = lever_arm.cross(thrust);

        ext_force.apply_force(thrust);
        ext_torque.apply_torque(torque);
    }
}

/// Evaluates multi-probe Archimedes hull buoyancy and water damping.
fn accumulate_buoyancy_system(
    water: Res<WaterSurfaceResource>,
    mut query: Query<
        (
            &Transform,
            &LinearVelocity,
            &AngularVelocity,
            &mut HullBuoyancyComponent,
            &mut ExternalForce,
            &mut ExternalTorque,
        ),
        With<EfoilRigidBody>,
    >,
) {
    for (transform, linear_vel, angular_vel, mut buoyancy, mut ext_force, mut ext_torque) in
        &mut query
    {
        let board_pos = transform.translation;

        let buoyancy_result = calculate_hull_buoyancy(
            &buoyancy.config,
            board_pos,
            |local| transform.transform_point(local),
            water.wave_height_fn,
            SEAWATER_DENSITY,
        );

        buoyancy.current_submerged_ratio = buoyancy_result.submerged_ratio;
        buoyancy.last_buoyant_force = buoyancy_result.total_force;
        buoyancy.last_buoyant_torque = buoyancy_result.total_torque;

        ext_force.apply_force(buoyancy_result.total_force);
        ext_torque.apply_torque(buoyancy_result.total_torque);

        // Water surface hull damping (prevents perpetual oscillation when floating)
        if buoyancy_result.submerged_ratio > 0.001 {
            let linear_damping_force =
                -linear_vel.0 * (buoyancy.config.linear_damping * buoyancy_result.submerged_ratio);
            let angular_damping_torque = -angular_vel.0
                * (buoyancy.config.angular_damping * buoyancy_result.submerged_ratio);

            ext_force.apply_force(linear_damping_force);
            ext_torque.apply_torque(angular_damping_torque);
        }
    }
}

/// Evaluates hydrofoil wing lift, induced & profile drag, mast keel resistance, and surface breach.
fn accumulate_hydrofoil_system(
    water: Res<WaterSurfaceResource>,
    mut query: Query<
        (
            &Transform,
            &LinearVelocity,
            &AngularVelocity,
            &mut HydrofoilWingsComponent,
            &mut ExternalForce,
            &mut ExternalTorque,
        ),
        With<EfoilRigidBody>,
    >,
) {
    for (transform, linear_vel, angular_vel, mut wings, mut ext_force, mut ext_torque) in &mut query
    {
        let forward = transform.forward().as_vec3();
        let up = transform.up().as_vec3();
        let right = transform.right().as_vec3();

        // 1. Front Wing (Main Lift Wing)
        let front_world = transform.transform_point(wings.front_wing_local_pos);
        let water_y_front = (water.wave_height_fn)(front_world.x, front_world.z);
        let front_depth = water_y_front - front_world.y;
        let is_breached = front_depth < 0.0;
        wings.front_wing_breached = is_breached;

        let v_front = linear_vel.0 + angular_vel.0.cross(front_world - transform.translation);

        let (lift_front, drag_front) = if is_breached {
            // Ventilated! Lift collapses instantly in air
            calculate_wing_forces(
                &wings.front_wing_profile,
                v_front,
                up,
                forward,
                1.225, // Air density
                0.0,
            )
        } else {
            calculate_wing_forces(
                &wings.front_wing_profile,
                v_front,
                up,
                forward,
                SEAWATER_DENSITY,
                1.0,
            )
        };

        // 2. Stabilizer Wing (Pitch Trim & Longitudinal Stability)
        let stab_world = transform.transform_point(wings.stabilizer_local_pos);
        let water_y_stab = (water.wave_height_fn)(stab_world.x, stab_world.z);
        let stab_submerged = if stab_world.y < water_y_stab {
            1.0
        } else {
            0.0
        };

        let v_stab = linear_vel.0 + angular_vel.0.cross(stab_world - transform.translation);
        let (lift_stab, drag_stab) = calculate_wing_forces(
            &wings.stabilizer_profile,
            v_stab,
            up,
            forward,
            SEAWATER_DENSITY,
            stab_submerged,
        );

        // 3. Mast Lateral Keel Resistance (opposes side-slip)
        let mast_center_world = transform.transform_point(Vec3::new(0.0, -0.45, -0.10));
        let water_y_mast = (water.wave_height_fn)(mast_center_world.x, mast_center_world.z);
        let mast_submersion =
            ((water_y_mast - (transform.translation.y - 0.85)) / 0.85).clamp(0.0, 1.0);
        let side_speed = linear_vel.0.dot(right);
        let mast_lateral_force =
            -right * (side_speed * wings.mast_lateral_damping * mast_submersion);

        // Calculate torques about center of mass
        let torque_front = (front_world - transform.translation).cross(lift_front + drag_front);
        let torque_stab = (stab_world - transform.translation).cross(lift_stab + drag_stab);
        let torque_mast = (mast_center_world - transform.translation).cross(mast_lateral_force);

        let total_foil_force = lift_front + drag_front + lift_stab + drag_stab + mast_lateral_force;
        let total_foil_torque = torque_front + torque_stab + torque_mast;

        wings.last_lift_force = lift_front + lift_stab;
        wings.last_drag_force = drag_front + drag_stab;

        if v_front.length_squared() > 0.1 {
            let flow_dir = v_front.normalize();
            let dot = forward.dot(flow_dir).clamp(-1.0, 1.0);
            wings.last_alpha_deg = dot.acos().to_degrees();
        }

        ext_force.apply_force(total_foil_force);
        ext_torque.apply_torque(total_foil_torque);
    }
}

/// Evaluates rider counter-balancing forces and torques:
/// F_rider = m_rider * (g - (omega x v))
/// tau_rider = r_rider x F_rider
fn accumulate_rider_dynamics_system(
    mut query: Query<
        (
            &Transform,
            &LinearVelocity,
            &AngularVelocity,
            &mut RiderCounterBalance,
            &mut ExternalForce,
            &mut ExternalTorque,
        ),
        With<EfoilRigidBody>,
    >,
) {
    for (transform, linear_vel, angular_vel, mut rider, mut ext_force, mut ext_torque) in &mut query
    {
        let v = linear_vel.0;
        let omega = angular_vel.0;

        // Centrifugal acceleration experienced by rider in turn: a_c = -(omega x v)
        let a_centrifugal = -omega.cross(v);
        let f_centrifugal = a_centrifugal * rider.rider_mass;

        // Rider gravity vector in world space
        let f_gravity = Vec3::new(0.0, -STANDARD_GRAVITY * rider.rider_mass, 0.0);

        // Total rider force vector acting on the deck
        let f_rider_total = f_gravity + f_centrifugal;

        // Stance position with dynamic pitch (fore/aft) and roll (lateral) weight shift
        let local_stance = rider.nominal_stance_pos
            + Vec3::new(
                rider.roll_lean * rider.max_roll_arm,
                0.0,
                rider.pitch_lean * rider.max_pitch_arm,
            );

        let stance_world = transform.transform_point(local_stance);
        let lever_arm = stance_world - transform.translation;

        // Torque produced by rider counter-balancing force about board center of mass
        let rider_torque = lever_arm.cross(f_rider_total);

        rider.last_centrifugal_force = f_centrifugal;
        rider.last_rider_force = f_rider_total;
        rider.last_rider_torque = rider_torque;

        // Centrifugal force pushes the board horizontally; rider torque controls pitch & roll balance
        ext_force.apply_force(f_centrifugal);
        ext_torque.apply_torque(rider_torque);
    }
}

/// Updates telemetry metrics for HUD and debug instruments.
fn update_telemetry_system(
    water: Res<WaterSurfaceResource>,
    mut query: Query<
        (
            &Transform,
            &LinearVelocity,
            &HullBuoyancyComponent,
            &HydrofoilWingsComponent,
            &RiderCounterBalance,
            &mut FlightTelemetry,
        ),
        With<EfoilRigidBody>,
    >,
) {
    for (transform, linear_vel, buoyancy, wings, rider, mut telemetry) in &mut query {
        let speed_mps = linear_vel.0.length();
        telemetry.speed_mps = speed_mps;
        telemetry.speed_kmh = mps_to_kmh(speed_mps);
        telemetry.speed_knots = mps_to_knots(speed_mps);

        let water_y = (water.wave_height_fn)(transform.translation.x, transform.translation.z);
        telemetry.ride_height = transform.translation.y - water_y;

        let front_world = transform.transform_point(wings.front_wing_local_pos);
        let water_y_front = (water.wave_height_fn)(front_world.x, front_world.z);
        telemetry.wing_depth = water_y_front - front_world.y;

        let (pitch, _yaw, roll) = transform.rotation.to_euler(EulerRot::XYZ);
        telemetry.pitch_deg = pitch.to_degrees();
        telemetry.roll_deg = roll.to_degrees();

        telemetry.lateral_g =
            rider.last_centrifugal_force.length() / (rider.rider_mass * STANDARD_GRAVITY).max(1.0);

        telemetry.state = if wings.front_wing_breached {
            FlightState::Breached
        } else if buoyancy.current_submerged_ratio < 0.05 && speed_mps > 3.0 {
            FlightState::Foiling
        } else if speed_mps > 2.0 {
            FlightState::Planing
        } else {
            FlightState::Floating
        };
    }
}
