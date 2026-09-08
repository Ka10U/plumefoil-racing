# GEMINI.md - Plumefoil Racing Project Guide

## 1. Project Overview & Vision

**Plumefoil Racing** is a high-performance, physics-authentic, 3D electric hydrofoil (e-foil) racing and simulation game built with **Rust** and the **Bevy Engine**.

### Core Objectives
1. **Compelling, Multi-Tiered Gameplay**: Deliver an intuitive yet deep "easy to learn, hard to master" flight experience across diverse game modes (Gate Slalom, Free Ride Exploration, Tricks & Style Score, Swell Pumping, and Battery Endurance).
2. **Realistic Hydrodynamics Simulation**: Faithful representation of efoil physics—hydrofoil lift/drag, Archimedes buoyancy, rider center-of-gravity weight distribution, motor thrust, and foil surface breaching/ventilation.
3. **Plume Brand Showcase & Real-World Efoil Rewards**: Authentically represent Plume efoils, components (boards, masts, wings, motors, batteries), and performance profiles. Offer a disruptive real-world reward system: granting real-world Plume efoils (2–5 per year) to top verified leaderboard champions.
4. **Diverse World Environments**: Diverse riding locations with varied water conditions, landscapes, currents, and challenges (Corsica Beach, Norway Fjord, Arctic Sea, Garda Lake, Raglan Beach, Seine River in Paris).
5. **Modern SynthWave Electro Ambient Soundtrack**: Immersive retro-futuristic synth pads, pulsating basslines, and chill ambient waves that dynamically react to riding speed and carving intensity.
6. **Cross-Platform Scalability**: First-class performance across **PC (Windows, macOS, Linux)** and **Mobile (Android & iOS)**, targeting solid 60 FPS on mobile GPUs.
7. **Cheat-Proof Competitive Integrity**: Deterministic headless simulation with input-log replay verification to guarantee unhackable leaderboards and protect physical efoil rewards.

---

## 2. Technology Stack & Key Dependencies

- **Programming Language**: Rust (2024 edition, stable toolchain).
- **Engine**: [Bevy Engine](https://bevyengine.org/) (0.15+).
  - ECS architecture for high cache efficiency and modular game systems.
- **Physics Engine**: `avian3d` (formerly `bevy_xpbd_3d`) or `bevy_rapier3d`.
  - Deterministic fixed-timestep simulation (`FixedUpdate`).
  - Custom hydrodynamics, wave buoyancy, lift, drag, and thrust force accumulators.
- **Input System**: `leafwing-input-manager`.
  - Unified input action maps for Keyboard/Mouse, Gamepads, and Mobile Touch Gestures.
  - Input logging & serialization for deterministic server-side replay verification.
- **Shaders / Water**: Custom WGSL shaders (Gerstner waves, low-overhead mobile PBR water, vertex displacement, depth-based color absorption, and foam).
- **Audio & Music**:
  - Spatial 3D sound (`bevy_kira_audio` or Bevy native audio) for motor RPM, foil whistling, spray, and hull slap.
  - Dynamic interactive SynthWave electro ambient music system (stems/layers fading with speed, carve angles, and flight state).
- **Anti-Cheat & Verification**:
  - Deterministic input stream recording (`InputLog`).
  - Headless physics validation runner re-executing player inputs to verify scores and lap times before leaderboard submission.
- **Asset Pipeline**:
  - Offline CAD (STEP) conversion to glTF 2.0 / GLB via FreeCAD / Blender / Python tooling.
  - PBR texture compression (KTX2 / Basis Universal) for mobile memory optimization.

---

## 3. Core Physics & Simulation Architecture

Efoiling is uniquely governed by dynamic equilibrium between **hydrodynamic lift**, **water surface interaction**, and **rider mass positioning**.

### 3.1 Force Balance (`sum of forces = m * a`)
At every fixed physics step, the following forces are evaluated on the efoil rigid body:
$$\vec{F}_{\text{total}} = \vec{F}_{\text{gravity}} + \vec{F}_{\text{buoyancy}} + \vec{F}_{\text{hull\_drag}} + \vec{F}_{\text{thrust}} + \vec{F}_{\text{front\_wing}} + \vec{F}_{\text{stab\_wing}} + \vec{F}_{\text{mast\_drag}}$$

$$\vec{\tau}_{\text{total}} = \sum (\vec{r}_i \times \vec{F}_i) + \vec{\tau}_{\text{rider\_torque}}$$

### 3.2 Key Physical Modules
1. **Multi-Point Buoyancy (Hull & Board)**:
   - Sample 4 to 8 probe points across the board hull against water surface elevation $h_{\text{water}}(x, z, t)$.
   - Submerged depth $d = \max(0.0, h_{\text{water}} - y_{\text{probe}})$.
   - Applies Archimedes buoyant force $\vec{F}_b = \rho_{\text{water}} \cdot V_{\text{submerged}} \cdot \vec{g}$ and hydrodynamic hull damping.
   - Water density $\rho_{\text{water}}$ adapts to location (e.g. $1025\,\text{kg/m}^3$ ocean vs. $997\,\text{kg/m}^3$ alpine freshwater lake like Garda).
2. **Hydrofoil Lift and Drag (Front & Stabilizer Wings)**:
   - Evaluated at wing chord positions submerged in water.
   - Relative velocity $\vec{v}_{\text{rel}} = \vec{v}_{\text{wing}} - \vec{v}_{\text{water\_current}}$ (supports river currents in Paris Seine!).
   - Angle of Attack ($\alpha$): Angle between chord line and velocity vector.
   - Lift: $L = \frac{1}{2} \rho_{\text{water}} v_{\text{rel}}^2 S C_L(\alpha)$
   - Induced & Profile Drag: $D = \frac{1}{2} \rho_{\text{water}} v_{\text{rel}}^2 S C_D(\alpha)$
   - Stall handling: When $\alpha > \alpha_{\text{stall}}$, lift drops sharply and drag spikes.
3. **Surface Breaching & Ventilation**:
   - If the front wing pierces the water surface ($y_{\text{wing}} > h_{\text{water}}$), foil lift immediately collapses ($C_L \to 0$, air entrainment).
   - Results in realistic nose drop / wipeout if uncorrected.
4. **Rider Weight Shift (Pitch & Roll Control)**:
   - The rider does not turn a rudder; they **shift their stance and center of mass (CoM)**.
   - Inputs:
     - Pitch axis: Shift weight forward (pushes nose down, decreases $\alpha$) or backward (pulls nose up, increases $\alpha$).
     - Roll axis: Shift weight heel-to-toe (induces bank angle, rolling the hydrofoil to carve turns).
   - Motor Throttle: Scalar trigger $[0.0, 1.0]$ controlling electric motor torque and propeller thrust.

---

## 4. World Environments & Riding Spots

Each location features unique visual styling, environmental audio, and hydrodynamics parameters:

1. **Corsica Beach (Mediterranean)**:
   - *Water*: Crystal-clear turquoise water, gentle swells, low chop.
   - *Landscape*: Sun-drenched granite rocks, sandy coves, pine trees.
   - *Gameplay*: Beginner-friendly, wide slalom gates, scenic cruising.
2. **Norway Fjord (Nordic Glass)**:
   - *Water*: Deep sheltered water, mirror-like glassy surface.
   - *Landscape*: Towering sheer vertical cliffs, misty waterfalls, dramatic shadows.
   - *Gameplay*: High-speed time trials, high precision carving with zero wave disturbance.
3. **Arctic Sea (Polar Expedition)**:
   - *Water*: Frigid, dark ocean water with floating ice floes and slush.
   - *Landscape*: Glaciers, icebergs, glowing Aurora Borealis night racing.
   - *Gameplay*: Extreme obstacle avoidance, freezing spray, technical navigation.
4. **Garda Lake (Alpine Freshwater)**:
   - *Water*: Freshwater ($\rho \approx 997\,\text{kg/m}^3$, requiring higher takeoff speed), choppy thermal wind waves.
   - *Landscape*: Dramatic Italian mountain ridges, lakeside historic villages.
   - *Gameplay*: Slalom agility, altitude thermals affecting chop.
5. **Raglan Beach (New Zealand Swell)**:
   - *Water*: Massive long-period ocean swells, peeling point-break waves.
   - *Landscape*: Black sand beaches, lush coastal hills, dramatic surf breaks.
   - *Gameplay*: Swell pumping without motor, wave riding, high-G bottom turns.
6. **Seine River in Paris (Urban Current)**:
   - *Water*: River current field ($\vec{v}_{\text{current}} = 1.0\text{--}2.5\,\text{m/s}$), wake chop from boats.
   - *Landscape*: Historic stone bridges, Eiffel Tower, Notre-Dame, urban quays.
   - *Gameplay*: Low bridge arch navigation, upstream vs. downstream speed tactics, barge obstacles.

---

## 5. Anti-Cheat & Physical Efoil Reward Verification

Because real Plume efoils (worth several thousand euros) are rewarded to top seasonal players:

1. **Deterministic Input Replays**:
   - The game client never reports just a final time or score.
   - The client records an authenticated **Input Stream** (seed, timestamped raw inputs: throttle, pitch, roll).
2. **Server-Side Replay Validation**:
   - A headless verification runner executes the exact same inputs in a deterministic physics loop.
   - If the simulated end state, checkpoints, or lap time mismatch the client's claim, the submission is rejected.
3. **Anomaly & Bot Detection**:
   - Telemetry profiling: detection of unnatural input frequencies (e.g., instant 0-to-100% stance snaps in $< 1\text{ms}$), memory hooks, and impossible accelerations.
4. **Community Ghost Verification**:
   - Top 10 leaderboard runs are publicly downloadable as ghost replays, enabling full community transparency and peer review.
5. **Account Verification**:
   - Strict anti-sybil and player identity verification required for prize-eligible tiers.

---

## 6. Repository & Architecture Layout

```
plumefoil-racing/
├── .cargo/
│   └── config.toml                  # Fast-compilation flags, cross-compile targets
├── assets/
│   ├── models/                      # glTF/GLB models (boards, wings, masts, riders, buoys)
│   ├── shaders/                     # WGSL shaders (water, foam, sky, UI)
│   ├── textures/                    # PBR textures (compressed with KTX2)
│   ├── music/                       # SynthWave electro ambient tracks (stems)
│   └── audio/                       # Sound effects (motor hum, water slap, wind)
├── crates/ / src/
│   ├── main.rs                      # App initialization & plugin registration
│   ├── core/                        # Game states, time scales, math utilities
│   ├── physics/
│   │   ├── hydrodynamics.rs         # Lift, drag, foil polar curves, stall
│   │   ├── buoyancy.rs              # Hull probe sampling, displaced volume
│   │   ├── propulsion.rs            # Motor, propeller thrust, battery curve
│   │   └── forces.rs                # Force accumulator system
│   ├── efoil/
│   │   ├── components.rs            # Efoil, Board, Mast, Wing, Battery specs
│   │   └── assembly.rs              # Spawning & assembling efoil entities
│   ├── rider/
│   │   ├── stance.rs                # Weight distribution, CoM offset, lean angles
│   │   └── animation.rs             # Procedural / rigged rider posture
│   ├── water/
│   │   ├── gerstner.rs              # CPU analytical wave height query
│   │   ├── material.rs              # Custom WGSL water material & pipeline
│   │   └── currents.rs              # Vector fields for river currents & swells
│   ├── world/
│   │   ├── environments.rs          # Environment presets (Corsica, Norway, Arctic, etc.)
│   │   └── obstacles.rs             # Icebergs, bridges, rocks, barges
│   ├── input/
│   │   ├── actions.rs               # Action definitions (Throttle, Pitch, Roll, Camera)
│   │   ├── desktop.rs               # Keyboard / Gamepad bindings
│   │   ├── mobile.rs                # On-screen virtual joystick & throttle slider
│   │   └── replay.rs                # Deterministic input recorder & serializer
│   ├── modes/
│   │   ├── race.rs                  # Gate slalom, lap timers, sector splits
│   │   ├── freeride.rs              # Open exploration, chill cruising
│   │   ├── tricks.rs                # Style scoring, carving intensity, wingtip cuts
│   │   └── swell_pumping.rs         # Motor-off wave riding & kinetic pumping
│   ├── anti_cheat/
│   │   ├── replay_validator.rs      # Headless physics run validation
│   │   └── anomaly_detection.rs     # Heuristics against bots & memory injections
│   ├── audio/
│   │   ├── sfx.rs                   # Motor whine, spray, hull slap, foil whistling
│   │   └── synthwave.rs             # Dynamic SynthWave adaptive music controller
│   ├── ui/
│   │   ├── hud.rs                   # Telemetry: speed (km/h & knots), height, battery, throttle
│   │   ├── leaderboards.rs          # Verified rankings & ghost replay browser
│   │   └── menus.rs                 # Main menu, Plume garage, pause screen
│   └── assets/
│       └── cad_loader.rs            # Asset loader setup & glTF instantiation
├── tools/
│   ├── cad_converter/               # Scripts (Python / FreeCAD) to convert STEP -> glTF
│   └── validator_cli/               # Standalone CLI binary to re-validate replays on server
├── Implementation Plans/            # Granular milestone & feature implementation plans
├── Master_Implementation_Plan.md    # Global roadmap and project master plan
├── initial_description.md           # Original high-level requirements
└── Cargo.toml                       # Workspace manifest & dependencies
```

---

## 7. Development & Build Workflows

### 7.1 Prerequisites
- Rust 1.85+ (`rustup default stable`)
- Git repository: `https://github.com/Ka10U/plumefoil-racing.git`
- Target toolchains:
  - Desktop: `x86_64-pc-windows-msvc`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`
  - Mobile: `cargo-apk` + Android NDK, `cargo-lipo` / Xcode for iOS
  - Web: `wasm32-unknown-unknown` + `wasm-server-runner`

### 7.2 Fast Compile Workflow (Development)
In `.cargo/config.toml`, configure dynamic linking and the fast lld/mold linker:
```toml
[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"

# Enable bevy dynamic linking for dev builds
[dependencies]
bevy = { version = "0.15", features = ["dynamic_linking"] }
```

### 7.3 Common Commands
```bash
# Run desktop build in debug mode with fast linking
cargo run

# Run with optimization level 2 for physics and hydrodynamics profiling
cargo run --release

# Run headless replay validation test suite
cargo test --package anti_cheat

# Run web build (for quick browser testing)
cargo run --target wasm32-unknown-unknown
```

---

## 8. Coding Standards & Conventions

1. **ECS System Organization**:
   - Keep systems focused and single-purpose.
   - Execute physics calculations strictly in `FixedUpdate` to ensure frame-rate independence and replay determinism.
   - Separate visual interpolation from physics state.
2. **Data-Driven Configuration**:
   - Store efoil component specs (Plume board volumes, wing surface areas, aspect ratios, battery capacities) and environment parameters in declarative Rust structs or Ron/JSON asset files.
3. **Replay Determinism**:
   - Never use non-deterministic random number generators inside physics or scoring systems (`rand::thread_rng()` is forbidden in `FixedUpdate`; use seeded PRNGs like `ChaCha8Rng` keyed to the match seed).
4. **No Hidden Panics**:
   - Use `Result` and `Option` gracefully. Never `unwrap()` or `expect()` in per-frame update systems.
5. **Mobile First Performance**:
   - Avoid dynamic allocations in per-frame loops.
   - Keep mesh poly counts reasonable (efoil visual model < 25k triangles, collision hull < 500 triangles).
   - Synchronize the CPU wave math (Gerstner) with GPU WGSL so CPU buoyancy precisely matches visual water surface without CPU-GPU readbacks.

---

## 9. AI Assistant Instructions

When working in this codebase:
- Always reference [Master_Implementation_Plan.md](file:///c:/05%20Vibe/Plumefoil%20Racing/Master_Implementation_Plan.md) for architectural alignment.
- Implement features via detailed, numbered plans in `Implementation Plans/`.
- Prioritize realistic hydrodynamics first: if the flight feel isn't authentic and satisfying, graphics won't save the game.
- Maintain full cross-platform compatibility across Desktop and Mobile at every step.
- Ensure all physics updates remain strictly deterministic for anti-cheat verification.
