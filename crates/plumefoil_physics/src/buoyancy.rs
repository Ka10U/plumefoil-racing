use glam::Vec3;
use plumefoil_core::constants::STANDARD_GRAVITY;

/// A single buoyancy sampling probe positioned on the hull.
#[derive(Debug, Clone, Copy)]
pub struct HullProbe {
    /// Local position offset from board rigid body origin (meters).
    pub local_position: Vec3,
    /// Displaced volume associated with this probe point when fully submerged (cubic meters).
    pub max_volume: f32,
    /// Probe vertical height/thickness over which buoyancy transitions (meters).
    pub probe_height: f32,
}

/// Hull buoyancy configuration composed of multiple sampling probes.
#[derive(Debug, Clone)]
pub struct HullBuoyancyConfig {
    pub probes: Vec<HullProbe>,
    /// Linear damping coefficient when hull is in contact with water (N*s/m).
    pub linear_damping: f32,
    /// Angular damping coefficient when hull is in contact with water (N*m*s/rad).
    pub angular_damping: f32,
}

impl Default for HullBuoyancyConfig {
    fn default() -> Self {
        // Standard Plume board: ~100 Liters (0.100 m^3) distributed across 6 probes
        let vol_per_probe = 0.100 / 6.0;
        let h = 0.12; // 12 cm board thickness

        let probes = vec![
            // Front (Nose) Left & Right
            HullProbe {
                local_position: Vec3::new(-0.25, -0.05, 0.65),
                max_volume: vol_per_probe,
                probe_height: h,
            },
            HullProbe {
                local_position: Vec3::new(0.25, -0.05, 0.65),
                max_volume: vol_per_probe,
                probe_height: h,
            },
            // Center (Mid-board) Left & Right
            HullProbe {
                local_position: Vec3::new(-0.30, -0.05, 0.00),
                max_volume: vol_per_probe,
                probe_height: h,
            },
            HullProbe {
                local_position: Vec3::new(0.30, -0.05, 0.00),
                max_volume: vol_per_probe,
                probe_height: h,
            },
            // Rear (Tail) Left & Right
            HullProbe {
                local_position: Vec3::new(-0.25, -0.05, -0.65),
                max_volume: vol_per_probe,
                probe_height: h,
            },
            HullProbe {
                local_position: Vec3::new(0.25, -0.05, -0.65),
                max_volume: vol_per_probe,
                probe_height: h,
            },
        ];

        Self {
            probes,
            linear_damping: 150.0,
            angular_damping: 80.0,
        }
    }
}

/// Result of evaluating buoyancy across all hull probes.
#[derive(Debug, Clone, Copy, Default)]
pub struct BuoyancyResult {
    /// Total upward buoyant force vector in world space (Newtons).
    pub total_force: Vec3,
    /// Total torque around board center of mass in world space (N*m).
    pub total_torque: Vec3,
    /// Overall submerged fraction of the board [0.0 = completely airborne, 1.0 = fully submerged].
    pub submerged_ratio: f32,
}

/// Evaluates buoyant forces across all hull probes against water surface elevation.
///
/// `board_transform_fn`: Closure mapping a local probe point to its world-space position.
/// `water_height_fn`: Closure returning the water surface elevation $y = h(x, z)$ at any world coordinate.
pub fn calculate_hull_buoyancy<F, W>(
    config: &HullBuoyancyConfig,
    board_com_world: Vec3,
    to_world_fn: F,
    water_height_fn: W,
    water_density: f32,
) -> BuoyancyResult
where
    F: Fn(Vec3) -> Vec3,
    W: Fn(f32, f32) -> f32,
{
    let mut total_force = Vec3::ZERO;
    let mut total_torque = Vec3::ZERO;
    let mut total_submerged_volume = 0.0;
    let mut total_max_volume = 0.0;

    for probe in &config.probes {
        let probe_world = to_world_fn(probe.local_position);
        let water_y = water_height_fn(probe_world.x, probe_world.z);
        let depth = water_y - (probe_world.y - probe.probe_height * 0.5);

        total_max_volume += probe.max_volume;

        if depth > 0.0 {
            let immersion = (depth / probe.probe_height).clamp(0.0, 1.0);
            let submerged_vol = probe.max_volume * immersion;
            total_submerged_volume += submerged_vol;

            let buoyant_magnitude = water_density * submerged_vol * STANDARD_GRAVITY;
            let force = Vec3::Y * buoyant_magnitude;

            // Torque = r x F where r is lever arm from center of mass to probe
            let lever_arm = probe_world - board_com_world;
            let torque = lever_arm.cross(force);

            total_force += force;
            total_torque += torque;
        }
    }

    let submerged_ratio = if total_max_volume > 0.0 {
        total_submerged_volume / total_max_volume
    } else {
        0.0
    };

    BuoyancyResult {
        total_force,
        total_torque,
        submerged_ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plumefoil_core::constants::SEAWATER_DENSITY;

    #[test]
    fn test_fully_submerged_buoyancy() {
        let config = HullBuoyancyConfig::default();
        let total_vol: f32 = config.probes.iter().map(|p| p.max_volume).sum();

        // Board resting 1 meter underwater
        let result = calculate_hull_buoyancy(
            &config,
            Vec3::new(0.0, -1.0, 0.0),
            |local| local + Vec3::new(0.0, -1.0, 0.0),
            |_x, _z| 0.0,
            SEAWATER_DENSITY,
        );

        let expected_force = SEAWATER_DENSITY * total_vol * STANDARD_GRAVITY;
        assert!((result.total_force.y - expected_force).abs() < 1.0);
        assert!((result.submerged_ratio - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_fully_airborne_produces_zero_buoyancy() {
        let config = HullBuoyancyConfig::default();
        let result = calculate_hull_buoyancy(
            &config,
            Vec3::new(0.0, 2.0, 0.0),
            |local| local + Vec3::new(0.0, 2.0, 0.0),
            |_x, _z| 0.0,
            SEAWATER_DENSITY,
        );

        assert_eq!(result.total_force, Vec3::ZERO);
        assert_eq!(result.total_torque, Vec3::ZERO);
        assert_eq!(result.submerged_ratio, 0.0);
    }
}
