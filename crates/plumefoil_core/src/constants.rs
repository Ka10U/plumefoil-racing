//! Physical and environmental constants for Plumefoil Racing.

/// Standard gravitational acceleration on Earth (m/s^2).
pub const STANDARD_GRAVITY: f32 = 9.80665;

/// Density of standard seawater at 15°C (kg/m^3).
pub const SEAWATER_DENSITY: f32 = 1025.0;

/// Density of fresh water at 20°C (kg/m^3).
pub const FRESHWATER_DENSITY: f32 = 997.0;

/// Density of sea-level air at 15°C (kg/m^3).
pub const AIR_DENSITY: f32 = 1.225;

/// Kinematic viscosity of water at 20°C (m^2/s).
pub const WATER_KINEMATIC_VISCOSITY: f32 = 1.004e-6;

/// Conversion factor from meters per second to knots.
pub const MPS_TO_KNOTS: f32 = 1.94384;

/// Conversion factor from knots to meters per second.
pub const KNOTS_TO_MPS: f32 = 1.0 / MPS_TO_KNOTS;

/// Conversion factor from meters per second to kilometers per hour.
pub const MPS_TO_KMH: f32 = 3.6;

/// Conversion factor from kilometers per hour to meters per second.
pub const KMH_TO_MPS: f32 = 1.0 / MPS_TO_KMH;

/// Fixed physics simulation timestep frequency in Hertz.
pub const FIXED_PHYSICS_HZ: f64 = 60.0;

/// Fixed physics simulation delta time in seconds.
pub const FIXED_PHYSICS_DT: f32 = (1.0 / FIXED_PHYSICS_HZ) as f32;

#[inline]
pub fn mps_to_knots(mps: f32) -> f32 {
    mps * MPS_TO_KNOTS
}

#[inline]
pub fn knots_to_mps(knots: f32) -> f32 {
    knots * KNOTS_TO_MPS
}

#[inline]
pub fn mps_to_kmh(mps: f32) -> f32 {
    mps * MPS_TO_KMH
}

#[inline]
pub fn kmh_to_mps(kmh: f32) -> f32 {
    kmh * KMH_TO_MPS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_conversions() {
        let mps = 10.0;
        let kmh = mps_to_kmh(mps);
        assert!((kmh - 36.0).abs() < 1e-4);
        assert!((kmh_to_mps(kmh) - mps).abs() < 1e-4);

        let knots = mps_to_knots(mps);
        assert!((knots - 19.4384).abs() < 1e-3);
        assert!((knots_to_mps(knots) - mps).abs() < 1e-3);
    }
}
