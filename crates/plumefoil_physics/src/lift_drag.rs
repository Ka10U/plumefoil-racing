//! Hydrofoil wing lift and drag models.

use glam::Vec3;

/// Hydrofoil wing profile characteristics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WingProfile {
    /// Planform surface area in square meters (e.g. 0.08 m^2 for 800 cm^2 Plume wing).
    pub surface_area: f32,
    /// Wing aspect ratio (span^2 / surface_area).
    pub aspect_ratio: f32,
    /// Zero-lift profile drag coefficient (parasitic drag).
    pub cd0: f32,
    /// Lift curve slope in 1/radians (typically ~5.5 to 6.2 for hydrofoils).
    pub lift_slope: f32,
    /// Angle of attack at zero lift in radians.
    pub alpha_zero_lift: f32,
    /// Positive stall angle in radians (typically ~0.26 rad / 15 deg).
    pub stall_angle_pos: f32,
    /// Negative stall angle in radians.
    pub stall_angle_neg: f32,
    /// Oswald efficiency factor (typically 0.75 - 0.90 for hydrofoils).
    pub oswald_efficiency: f32,
}

impl Default for WingProfile {
    fn default() -> Self {
        Self {
            surface_area: 0.08, // 800 cm^2
            aspect_ratio: 6.5,
            cd0: 0.015,
            lift_slope: 5.5,
            alpha_zero_lift: -0.035,  // ~ -2 deg camber
            stall_angle_pos: 0.2618,  // +15 deg
            stall_angle_neg: -0.2094, // -12 deg
            oswald_efficiency: 0.85,
        }
    }
}

/// Computes lift coefficient $C_L$ with progressive post-stall degradation.
pub fn calculate_lift_coefficient(profile: &WingProfile, alpha: f32) -> f32 {
    let effective_alpha = alpha - profile.alpha_zero_lift;

    if alpha > profile.stall_angle_pos {
        // Post-stall lift decay
        let stall_overshoot = alpha - profile.stall_angle_pos;
        let cl_max = profile.lift_slope * (profile.stall_angle_pos - profile.alpha_zero_lift);
        cl_max * (-stall_overshoot * 4.0).exp().max(0.2)
    } else if alpha < profile.stall_angle_neg {
        // Negative post-stall decay
        let stall_overshoot = profile.stall_angle_neg - alpha;
        let cl_min = profile.lift_slope * (profile.stall_angle_neg - profile.alpha_zero_lift);
        cl_min * (-stall_overshoot * 4.0).exp().max(0.2)
    } else {
        // Linear attached flow regime
        profile.lift_slope * effective_alpha
    }
}

/// Computes total drag coefficient $C_D = C_{D0} + C_{D,\text{induced}}$.
pub fn calculate_drag_coefficient(profile: &WingProfile, cl: f32, alpha: f32) -> f32 {
    let induced_drag =
        (cl * cl) / (std::f32::consts::PI * profile.aspect_ratio * profile.oswald_efficiency);
    let mut cd = profile.cd0 + induced_drag;

    // Additional drag penalty in stall regime
    if alpha > profile.stall_angle_pos {
        let delta = alpha - profile.stall_angle_pos;
        cd += delta * 1.5;
    } else if alpha < profile.stall_angle_neg {
        let delta = profile.stall_angle_neg - alpha;
        cd += delta * 1.5;
    }

    cd
}

/// Evaluates hydrodynamic lift and drag forces produced by a wing submerged in fluid.
///
/// Returns `(lift_force, drag_force)` vectors in Newtons.
pub fn calculate_wing_forces(
    profile: &WingProfile,
    relative_velocity: Vec3,
    wing_normal: Vec3,
    chord_direction: Vec3,
    fluid_density: f32,
    submerged_ratio: f32,
) -> (Vec3, Vec3) {
    if submerged_ratio <= 0.0 || relative_velocity.length_squared() < 0.01 {
        return (Vec3::ZERO, Vec3::ZERO);
    }

    let speed = relative_velocity.length();
    let flow_dir = relative_velocity / speed;

    // Angle of attack: angle between chord line and oncoming flow
    // When flow is parallel to chord: alpha = 0
    let dot = chord_direction.dot(flow_dir).clamp(-1.0, 1.0);
    let mut alpha = dot.acos();

    // Determine sign: if flow comes from under the foil, alpha is positive
    if wing_normal.dot(flow_dir) > 0.0 {
        alpha = -alpha;
    }

    let cl = calculate_lift_coefficient(profile, alpha);
    let cd = calculate_drag_coefficient(profile, cl, alpha);

    let dynamic_pressure = 0.5 * fluid_density * speed * speed;
    let effective_area = profile.surface_area * submerged_ratio.clamp(0.0, 1.0);

    let lift_magnitude = dynamic_pressure * effective_area * cl;
    let drag_magnitude = dynamic_pressure * effective_area * cd;

    // Lift direction is perpendicular to flow direction in the plane of the wing normal
    let span_dir = flow_dir.cross(wing_normal).normalize_or_zero();
    let lift_dir = span_dir.cross(flow_dir).normalize_or_zero();

    let lift_force = lift_dir * lift_magnitude;
    let drag_force = -flow_dir * drag_magnitude;

    (lift_force, drag_force)
}

#[cfg(test)]
mod tests {
    use super::*;
    use plumefoil_core::constants::SEAWATER_DENSITY;

    #[test]
    fn test_lift_at_cruise() {
        let profile = WingProfile::default();
        let vel = Vec3::new(0.0, 0.0, 8.0); // 8 m/s (~15.5 knots)
        let chord = Vec3::new(0.0, 0.0, 1.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);

        let (lift, drag) =
            calculate_wing_forces(&profile, vel, normal, chord, SEAWATER_DENSITY, 1.0);

        // At 8 m/s with small camber, lift should easily support ~80-100 kg weight (780-980 N)
        assert!(lift.length() > 300.0, "Lift must be sufficient for flight");
        assert!(
            drag.length() > 0.0 && drag.length() < lift.length(),
            "L/D ratio should be positive and realistic"
        );
    }

    #[test]
    fn test_breach_produces_zero_submerged_force() {
        let profile = WingProfile::default();
        let vel = Vec3::new(0.0, 0.0, 10.0);
        let chord = Vec3::new(0.0, 0.0, 1.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);

        let (lift, drag) =
            calculate_wing_forces(&profile, vel, normal, chord, SEAWATER_DENSITY, 0.0);
        assert_eq!(lift, Vec3::ZERO);
        assert_eq!(drag, Vec3::ZERO);
    }
}
