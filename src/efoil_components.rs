//! ECS Components and resources for the physical efoil assembly, hydrodynamics,
//! propulsion, rider counter-balancing, and telemetry.

use bevy::prelude::*;
use plumefoil_physics::buoyancy::HullBuoyancyConfig;
use plumefoil_physics::lift_drag::WingProfile;
use plumefoil_physics::thrust::MotorConfig;

/// Marker component for the physical efoil rigid body root entity.
#[derive(Component, Debug, Reflect)]
pub struct EfoilRigidBody;

/// Multi-point hull buoyancy configuration and state.
#[derive(Component, Debug)]
pub struct HullBuoyancyComponent {
    pub config: HullBuoyancyConfig,
    pub current_submerged_ratio: f32,
    pub last_buoyant_force: Vec3,
    pub last_buoyant_torque: Vec3,
}

impl Default for HullBuoyancyComponent {
    fn default() -> Self {
        Self {
            config: HullBuoyancyConfig::default(),
            current_submerged_ratio: 1.0,
            last_buoyant_force: Vec3::ZERO,
            last_buoyant_torque: Vec3::ZERO,
        }
    }
}

/// Underwater hydrofoil wings configuration and aerodynamic state.
#[derive(Component, Debug)]
pub struct HydrofoilWingsComponent {
    pub front_wing_profile: WingProfile,
    pub stabilizer_profile: WingProfile,
    /// Front wing hydrodynamic center offset from board origin (meters).
    pub front_wing_local_pos: Vec3,
    /// Stabilizer hydrodynamic center offset from board origin (meters).
    pub stabilizer_local_pos: Vec3,
    /// Resistance coefficient against sideways mast drift (N*s/m).
    pub mast_lateral_damping: f32,
    /// Last computed front wing lift force (world space).
    pub last_lift_force: Vec3,
    /// Last computed total foil drag force (world space).
    pub last_drag_force: Vec3,
    /// Last angle of attack in degrees.
    pub last_alpha_deg: f32,
    /// Whether front wing has breached the water surface.
    pub front_wing_breached: bool,
}

impl Default for HydrofoilWingsComponent {
    fn default() -> Self {
        Self {
            // Front wing: 800 cm^2 high-lift wing
            front_wing_profile: WingProfile {
                surface_area: 0.08,
                aspect_ratio: 6.5,
                cd0: 0.012,
                lift_slope: 5.8,
                alpha_zero_lift: -0.035,
                stall_angle_pos: 0.2618,
                stall_angle_neg: -0.2094,
                oswald_efficiency: 0.88,
            },
            // Stabilizer wing: 220 cm^2 pitch trim downforce wing
            stabilizer_profile: WingProfile {
                surface_area: 0.022,
                aspect_ratio: 5.0,
                cd0: 0.015,
                lift_slope: 5.2,
                alpha_zero_lift: 0.0,
                stall_angle_pos: 0.22,
                stall_angle_neg: -0.22,
                oswald_efficiency: 0.85,
            },
            front_wing_local_pos: Vec3::new(0.0, -0.85, 0.15),
            stabilizer_local_pos: Vec3::new(0.0, -0.84, -0.65),
            mast_lateral_damping: 600.0,
            last_lift_force: Vec3::ZERO,
            last_drag_force: Vec3::ZERO,
            last_alpha_deg: 0.0,
            front_wing_breached: false,
        }
    }
}

/// Electric motor and propeller propulsion configuration.
#[derive(Component, Debug)]
pub struct EfoilPropulsionComponent {
    pub config: MotorConfig,
    /// Target throttle commanded by input [0.0, 1.0].
    pub target_throttle: f32,
    /// Smoothed active throttle [0.0, 1.0].
    pub current_throttle: f32,
    /// Motor propeller hub position in local board space.
    pub motor_local_pos: Vec3,
    /// Last applied thrust force vector in world space.
    pub last_thrust_force: Vec3,
}

impl Default for EfoilPropulsionComponent {
    fn default() -> Self {
        Self {
            config: MotorConfig {
                max_static_thrust: 480.0,
                max_pitch_speed: 15.0,
                response_time_constant: 0.15,
            },
            target_throttle: 0.0,
            current_throttle: 0.0,
            motor_local_pos: Vec3::new(0.0, -0.75, -0.20),
            last_thrust_force: Vec3::ZERO,
        }
    }
}

/// Rider counter-balancing dynamics model.
///
/// Models the rider as an active force vector balancing the unstable equilibrium of foil flight:
/// F_rider = m_rider * (g - (omega x v))
/// tau_rider = r_rider x F_rider
#[derive(Component, Debug)]
pub struct RiderCounterBalance {
    /// Rider body mass in kg (~75 kg).
    pub rider_mass: f32,
    /// Nominal stance center on the board deck (local coordinates).
    pub nominal_stance_pos: Vec3,
    /// Pitch lean input [-1.0, 1.0] (-1 = lean forward/nose down, +1 = lean back/nose up).
    pub pitch_lean: f32,
    /// Roll lean input [-1.0, 1.0] (-1 = lean left/heel, +1 = lean right/toe).
    pub roll_lean: f32,
    /// Maximum longitudinal foot pressure / CoM shift lever arm (meters).
    pub max_pitch_arm: f32,
    /// Maximum lateral heel-toe pressure / CoM shift lever arm (meters).
    pub max_roll_arm: f32,
    /// Last computed centrifugal force vector in world space.
    pub last_centrifugal_force: Vec3,
    /// Last computed total rider force vector (gravity + centrifugal).
    pub last_rider_force: Vec3,
    /// Last computed rider counter-balancing torque about board CoM.
    pub last_rider_torque: Vec3,
}

impl Default for RiderCounterBalance {
    fn default() -> Self {
        Self {
            rider_mass: 75.0,
            nominal_stance_pos: Vec3::new(0.0, 0.08, -0.05),
            pitch_lean: 0.0,
            roll_lean: 0.0,
            max_pitch_arm: 0.35,
            max_roll_arm: 0.16,
            last_centrifugal_force: Vec3::ZERO,
            last_rider_force: Vec3::ZERO,
            last_rider_torque: Vec3::ZERO,
        }
    }
}

/// Operational flight regime of the efoil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlightState {
    /// Board resting in water; buoyancy supports weight; high displacement drag.
    #[default]
    Floating,
    /// Transition speed; wing lift building; board beginning to plane.
    Planing,
    /// Full hydrofoil flight; hull completely clear of water; minimal drag.
    Foiling,
    /// Front wing has pierced water surface; lift collapsed; nose drops.
    Breached,
}

/// Real-time flight telemetry published to HUD and recording streams.
#[derive(Component, Debug, Default)]
pub struct FlightTelemetry {
    pub speed_mps: f32,
    pub speed_kmh: f32,
    pub speed_knots: f32,
    /// Clearance of the board bottom above water surface (meters).
    pub ride_height: f32,
    /// Submersion depth of front wing (positive = underwater, negative = breached).
    pub wing_depth: f32,
    pub pitch_deg: f32,
    pub roll_deg: f32,
    pub lateral_g: f32,
    pub state: FlightState,
}

/// Resource defining the analytical water surface elevation $y = h(x, z)$.
#[derive(Resource)]
pub struct WaterSurfaceResource {
    /// Analytical wave query closure or parameters.
    pub wave_height_fn: fn(f32, f32) -> f32,
}

impl Default for WaterSurfaceResource {
    fn default() -> Self {
        Self {
            wave_height_fn: |_x, _z| 0.0, // Baseline calm flat water
        }
    }
}
