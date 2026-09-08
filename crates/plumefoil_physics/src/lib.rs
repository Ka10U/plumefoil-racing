//! `plumefoil_physics` provides deterministic hydrodynamics, wing lift/drag,
//! Archimedes buoyancy, and electric motor propulsion calculations.

pub mod buoyancy;
pub mod lift_drag;
pub mod thrust;

pub use buoyancy::*;
pub use lift_drag::*;
pub use thrust::*;
