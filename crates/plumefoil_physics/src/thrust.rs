//! Electric motor and propeller propulsion model.

use glam::Vec3;

/// Electric powertrain and propeller configuration.
#[derive(Debug, Clone, Copy)]
pub struct MotorConfig {
    /// Maximum continuous thrust at zero advance speed in Newtons (e.g. 500 N for a 5kW efoil).
    pub max_static_thrust: f32,
    /// Maximum theoretical pitch speed in m/s (~15 m/s / ~30 knots).
    pub max_pitch_speed: f32,
    /// Motor throttle spin-up / response time constant (seconds).
    pub response_time_constant: f32,
}

impl Default for MotorConfig {
    fn default() -> Self {
        Self {
            max_static_thrust: 480.0, // ~480 N thrust
            max_pitch_speed: 15.0,    // ~54 km/h max pitch speed
            response_time_constant: 0.15,
        }
    }
}

/// Evaluates motor thrust given current throttle command and forward flow velocity.
///
/// Returns thrust force vector in Newtons aligned with the motor forward axis.
pub fn calculate_motor_thrust(
    config: &MotorConfig,
    throttle: f32,
    motor_forward: Vec3,
    advance_speed: f32,
) -> Vec3 {
    let clamped_throttle = throttle.clamp(0.0, 1.0);
    if clamped_throttle <= 0.0 {
        return Vec3::ZERO;
    }

    // Propeller advance efficiency curve: thrust declines as advance speed approaches pitch speed
    let speed_ratio = (advance_speed / config.max_pitch_speed).clamp(0.0, 1.0);
    let slip_factor = (1.0 - speed_ratio * 0.85).max(0.0);

    let thrust_magnitude = config.max_static_thrust * clamped_throttle * slip_factor;
    motor_forward * thrust_magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_thrust() {
        let config = MotorConfig::default();
        let thrust = calculate_motor_thrust(&config, 1.0, Vec3::Z, 0.0);
        assert!((thrust.z - config.max_static_thrust).abs() < 1e-4);
    }

    #[test]
    fn test_zero_throttle_produces_zero_thrust() {
        let config = MotorConfig::default();
        let thrust = calculate_motor_thrust(&config, 0.0, Vec3::Z, 0.0);
        assert_eq!(thrust, Vec3::ZERO);
    }
}
