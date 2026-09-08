# Implementation Plan 01: Hydrodynamic Physics & Buoyancy Prototype

> **Parent Document**: [Master Implementation Plan](file:///c:/05%20Vibe/Plumefoil%20Racing/Master_Implementation_Plan.md)  
> **Prerequisites**: [Implementation Plan 00](file:///c:/05%20Vibe/Plumefoil%20Racing/Implementation%20Plans/00_project_setup_and_toolchain.md) (Completed)  
> **Target Engine**: Rust + Bevy 0.15 + Avian3D 0.2  
> **Status**: Ready for Review  

---

## 1. Overview & Objectives

The primary objective of **Implementation Plan 01** is to transition Plumefoil Racing from a static visual mockup into a **fully dynamic, physics-authentic hydrofoil simulation**.

We will integrate Avian3D rigid body dynamics with our deterministic `plumefoil_physics` calculations inside Bevy's `FixedUpdate` schedule. Crucially, hydrofoil flight operates in a regime of **unstable equilibrium**: hydrodynamic lift acting at the underwater wing's center of pressure generates strong, nonlinear pitching moments that vary with velocity and angle of attack. Stable flight is achieved dynamically by modeling the **rider as an active counter-balancing force vector** (combining downward gravity and lateral centrifugal force during carve turns) and applying the resulting counter-balancing torques to the board.

### Key Objectives
1. **Avian3D Rigid Body Integration**: Configure the efoil assembly as a dynamic rigid body with physically accurate mass distribution ($\approx 35\,\text{kg}$ efoil hardware + $\approx 75\,\text{kg}$ rider = $110\,\text{kg}$ total).
2. **Multi-Point Buoyancy Accumulator**: Continuously sample 6 hull probes against water surface elevation, applying upward Archimedes buoyant forces and restoring stability torques with water damping.
3. **Hydrofoil Lift, Drag & Ventilation Accumulator**:
   - Compute relative flow velocity at the front wing and stabilizer chord centers.
   - Evaluate angle of attack ($\alpha$) and apply 3D lift and induced/profile drag.
   - Model mast lateral resistance (side-slip damping) to prevent sliding sideways like a boat keel.
   - Model surface breach / ventilation: when the front wing pierces the water surface, lift collapses instantly ($C_L \to 0$).
4. **Rider Counter-Balancing Dynamics (Gravity + Centrifugal Force)**:
   - Model the rider as an active physical force acting on the board deck:
     $$\vec{F}_{\text{rider}} = \vec{F}_{\text{gravity}} + \vec{F}_{\text{centrifugal}} = m_{\text{rider}} \left( \vec{g} - (\vec{\omega} \times \vec{v}) \right)$$
   - Calculate counter-balancing torques ($\vec{\tau}_{\text{rider}} = \vec{r}_{\text{rider}} \times \vec{F}_{\text{rider}}$) across pitch (longitudinal weight shift) and roll (heel/toe lateral shift) to maintain the narrow unstable equilibrium window required for flight.
5. **Propeller Propulsion**: Apply motor thrust aligned with the fuselage axis based on throttle input and propeller advance speed.
6. **Interactive Flight Controls**: Direct keyboard/gamepad controls for throttle and rider pitch/roll weight shifts to evaluate flight feel, equilibrium trimming, and carving stability.
7. **Visual Debug Gizmos**: Real-time 3D vector arrows and probe markers displaying buoyancy, lift, drag, thrust, rider force (gravity + centrifugal), and center of mass.

---

## 2. Physics & Force Architecture

In Bevy's `FixedUpdate` schedule (running at 60 Hz), the physics plugin evaluates all forces acting on the efoil entity before Avian3D integrates velocity and position:

```
                  +--------------------------------------------------------------+
                  |                   Bevy FixedUpdate (60 Hz)                   |
                  +--------------------------------------------------------------+
                                                 |
     +-------------------+-----------------------+-----------------------+-----------------------+
     |                   |                                               |                       |
     v                   v                                               v                       v
[1. Buoyancy]     [2. Hydrofoil Wings]                         [3. Rider Counter-Balance] [4. Motor Thrust]
- 6 Hull Probes   - Front wing lift/drag ($C_L, C_D$)          - Gravity: $m_r \vec{g}$   - Aligned with motor
- Displaced vol   - Stabilizer downforce/trim                  - Centrifugal:             - Slip ratio & advance
- Water damping   - Mast lateral keel resistance                 $-m_r (\vec{\omega}\times\vec{v})$ - Dynamic acceleration
- Surface breach  - Surface ventilation collapse ($C_L \to 0$) - Pitch & Roll Torques
     |                   |                                               |                       |
     +-------------------+-----------------------+-----------------------+-----------------------+
                                                 |
                                                 v
                                 [Sum of Forces & Torques]
                                 Applied to ExternalForce
                                                 |
                                                 v
                                 [Avian3D Physics Integration]
                                 Calculates new velocity, position,
                                 orientation, and angular momentum
```

### Mathematical Formulation of Rider Counter-Balance
1. **Unstable Equilibrium**: The front wing generates lift $L$ at distance $d_{\text{wing}}$ forward of the efoil center of mass, producing an upward pitching torque $\tau_{\text{wing}} = L \cdot d_{\text{wing}}$. Left unchecked, this pitches the nose up, increasing $\alpha$, which increases $L$ exponentially until stall or foil breach.
2. **Rider Counter-Torque**: The rider's weight acts at stance position $\vec{r}_{\text{rider}}$:
   $$\vec{F}_{\text{rider}} = m_{\text{rider}} \vec{g} - m_{\text{rider}} (\vec{\omega} \times \vec{v})$$
   $$\vec{\tau}_{\text{rider}} = (\vec{r}_{\text{rider}} - \vec{r}_{\text{CoM}}) \times \vec{F}_{\text{rider}}$$
3. **Pitch Trim**: The rider shifts their stance fore/aft ($\Delta z$) to counteract $\tau_{\text{wing}}$ plus stabilizer trim:
   $$\tau_{\text{pitch, total}} = \tau_{\text{wing}} + \tau_{\text{stab}} + (\Delta z_{\text{rider}} \cdot F_{\text{rider}, y}) \approx 0$$
4. **Carving & Centrifugal Equilibrium**: In a turn of radius $R$ at speed $v$, the board banks by angle $\phi$. The lateral centrifugal force $-m_{\text{rider}} (\vec{\omega} \times \vec{v})$ pushes outward, while gravity pulls downward. The rider shifts their stance laterally ($\Delta x_{\text{rider}}$) to match the coordinated bank angle $\tan(\phi) \approx \frac{\omega v}{g}$, achieving balanced carving.

### Component Data Model
- `EfoilRigidBody`: Marker component identifying the efoil root entity.
- `HullBuoyancy`: Stores the 6 sampling probes, displaced volumes, and linear/angular water damping parameters.
- `HydrofoilWings`: Stores front wing profile, stabilizer wing profile, chord vectors, and relative lever arms from CoM.
- `EfoilPropulsion`: Stores motor specs, max static thrust ($480\,\text{N}$), current throttle setting ($0.0\text{--}1.0$), and propeller pitch speed.
- `RiderCounterBalance`: Stores rider mass ($m_{\text{rider}} \approx 75\,\text{kg}$), stance baseline position, dynamic pitch lean ($[-1.0, 1.0]$), dynamic roll lean ($[-1.0, 1.0]$), and computes resulting gravity + centrifugal forces and counter-torques.
- `FlightTelemetry`: Live telemetry cache (speed in km/h & knots, ride height, wing depth, angle of attack $\alpha$, G-force, and flight state: *Floating*, *Planing*, *Foiling*, *Breached*).
- `WaterSurface`: Resource providing analytical water level $y = h(x, z)$ ($y = 0.0$ flat baseline, ready for Gerstner waves in Plan 04).

---

## 3. Step-by-Step Implementation Steps

### Step 1: Create Physics Plugin in Main Game (`src/physics_plugin.rs`)
- Implement `EfoilPhysicsPlugin` registered in `src/main.rs`.
- Configure Avian3D with `PhysicsPlugins::default()` and standard gravity ($\vec{g} = [0, -9.80665, 0]$).
- Register physics force accumulator systems into `FixedUpdate`.

### Step 2: Implement Component Schemas & Efoil Spawner
- Define components: `EfoilRigidBody`, `HullBuoyancy`, `HydrofoilWings`, `EfoilPropulsion`, `RiderCounterBalance`.
- Create assembly spawner replacing the static demo cuboids with an Avian3D rigid body:
  - `RigidBody::Dynamic`
  - `Collider::compound(...)` for board hull, mast, and wings.
  - `Mass(110.0)`
  - `ExternalForce::default()`
  - Initial position: floating at water surface level ($y \approx 0.1\,\text{m}$, speed $0\,\text{m/s}$).

### Step 3: Implement Buoyancy & Damping System
- Query each probe's world position via `Transform::transform_point`.
- Call `calculate_hull_buoyancy` from `plumefoil_physics::buoyancy`.
- Apply buoyant forces and restoring torques to `ExternalForce`.
- Apply vertical and rotational water damping proportional to hull submersion ratio.

### Step 4: Implement Wing Lift, Drag & Ventilation System
- Query rigid body `LinearVelocity`, `AngularVelocity`, and `Transform`.
- Calculate local velocity at the front wing: $\vec{v}_{\text{wing}} = \vec{v} + \vec{\omega} \times \vec{r}_{\text{wing}}$.
- Calculate local velocity at the stabilizer: $\vec{v}_{\text{stab}} = \vec{v} + \vec{\omega} \times \vec{r}_{\text{stab}}$.
- Call `calculate_wing_forces` from `plumefoil_physics::lift_drag`.
- Add mast lateral resistance force (side-slip damping: opposes velocity perpendicular to board centerline).
- Add ventilation check: if front wing $y > 0$, lift immediately zeroes out ($C_L \to 0$).
- Apply net hydrodynamic forces and torques to `ExternalForce`.

### Step 5: Implement Rider Counter-Balancing Dynamics (Gravity + Centrifugal Force)
- Query `LinearVelocity`, `AngularVelocity`, and `RiderCounterBalance`.
- Calculate centrifugal acceleration: $\vec{a}_{\text{centrifugal}} = -(\vec{\omega} \times \vec{v})$.
- Calculate total rider force vector:
  $$\vec{F}_{\text{rider}} = m_{\text{rider}} \vec{g} + m_{\text{rider}} \vec{a}_{\text{centrifugal}}$$
- Apply rider stance offset:
  - Longitudinal shift $\Delta z = \text{pitch\_input} \times \text{max\_pitch\_arm}$.
  - Lateral shift $\Delta x = \text{roll\_input} \times \text{max\_roll\_arm}$.
- Calculate rider counter-torque about efoil center of mass:
  $$\vec{\tau}_{\text{rider}} = (\vec{r}_{\text{stance}} + \Delta \vec{r} - \vec{r}_{\text{CoM}}) \times \vec{F}_{\text{rider}}$$
- Apply $\vec{F}_{\text{rider}}$ and $\vec{\tau}_{\text{rider}}$ directly to `ExternalForce`.

### Step 6: Implement Propulsion System
- Apply forward thrust from `plumefoil_physics::thrust` aligned with motor axis based on throttle and forward advance speed.

### Step 7: Interactive Input Handling & Camera Tracking
- Map keyboard controls:
  - `Space` / `W` (or Up Arrow): Throttle Up
  - `Shift` / `S` (or Down Arrow): Throttle Down
  - `Down Arrow` / `S`: Lean Back (Pitch Up / Counteract Nose Dive)
  - `Up Arrow` / `W`: Lean Forward (Pitch Down / Counteract Over-Climb)
  - `Left Arrow` / `A`: Lean Left (Heel turn / Bank Left)
  - `Right Arrow` / `D`: Lean Right (Toe turn / Bank Right)
  - `R`: Reset efoil to start position at water surface.
- Smooth third-person follow camera tracking the efoil with look-ahead and roll damping.

### Step 8: Visual Force Gizmos & HUD Telemetry
- Implement debug gizmos (toggleable via `F1`):
  - Buoyancy probes: colored spheres indicating submersion depth.
  - Lift force vector: green arrow at front wing.
  - Drag force vector: red arrow.
  - Thrust force vector: yellow arrow at motor.
  - Rider force vector (Gravity + Centrifugal): purple arrow originating from rider stance.
  - CoM position: white sphere.
- Update HUD overlay with real-time telemetry:
  - Speed in $\text{km/h}$ and $\text{knots}$.
  - Ride height above water (wing depth and board clearance).
  - Angle of attack ($\alpha$) and centrifugal G-force.
  - Current throttle percentage.
  - Flight regime: *Floating*, *Planing*, *Foiling*, or *Breached*.

---

## 4. Proposed Files

| File | Change | Description |
|---|---|---|
| `src/physics_plugin.rs` | [NEW] | Bevy plugin integrating Avian3D with `plumefoil_physics` force accumulators |
| `src/efoil_components.rs` | [NEW] | ECS components for efoil rigid body, wings, probes, motor, and rider counter-balance |
| `src/flight_debug.rs` | [NEW] | Debug gizmo visualizer for force vectors (lift, drag, thrust, rider force) and telemetry |
| `src/main.rs` | [MODIFY] | Register physics plugin, wire inputs, spawn physical efoil, and update camera chase |

---

## 5. Verification & Testing Plan

### Automated Tests
```bash
# Verify all unit tests continue to pass
cargo test --workspace

# Clippy check
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check
```

### Manual Interactive Verification
1. **Stationary Buoyancy Equilibrium**:
   - At launch ($0\,\text{km/h}$), the efoil sits stably in the water at natural waterline ($y \approx 0.1\,\text{m}$) with zero throttle.
   - Buoyancy balances total gravity ($110\,\text{kg} \times 9.81 = 1079\,\text{N}$) without sinking or launching into the sky.
2. **Takeoff Transition & Unstable Equilibrium Range**:
   - Apply throttle (`Space`). Board accelerates across water in displacement mode.
   - At $\approx 10\text{--}12\,\text{km/h}$, front wing lift builds.
   - Player adjusts pitch weight shift (`W` / `S`) to balance the wing's pitching moment, maintaining stable lift-off into foiling flight.
   - Hull leaves the water surface; hull drag drops to 0; board glides smoothly above the water.
3. **Flight Altitude Trimming**:
   - Slight lean back (`S`): increases angle of attack, climbing higher above water.
   - Slight lean forward (`W`): decreases angle of attack, lowering flight height.
4. **Carving & Centrifugal Force Balance**:
   - Press `A` / `D` to initiate a carved turn.
   - Observe purple rider force vector tilting dynamically as centrifugal force pushes outward.
   - Notice how leaning into the turn achieves coordinated banking balance.
5. **Surface Breach & Recovery**:
   - Lean back hard to force the front wing out of the water ($y > 0$).
   - Confirm lift collapses and board drops back down onto the water surface with hull damping.
6. **Reset (`R`)**:
   - Pressing `R` instantly resets the efoil to starting position and zero velocity.
