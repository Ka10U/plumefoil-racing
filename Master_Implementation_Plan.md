# Plumefoil Racing - Master Implementation Plan

> **Document Status**: Active Master Roadmap  
> **Target Engine**: Rust + Bevy Engine (0.15+)  
> **Platforms**: PC (Windows, macOS, Linux) & Mobile (Android, iOS)  
> **Source Control**: [GitHub Repository: Ka10U/plumefoil-racing](https://github.com/Ka10U/plumefoil-racing)  
> **Brand Partner**: Plume Efoils  

---

## 1. Executive Summary & Project Vision

**Plumefoil Racing** is a 3D electric hydrofoil (e-foil) racing and simulation game. The game captures the authentic physical sensation of hydrofoil flight—silent levitation over water, carve dynamics, wake riding, and high-speed slalom racing—with accessible, engaging arcade and simulation gameplay.

### Core Pillars
1. **Physical Authenticity**: Real physics-driven behavior ($F = m \cdot a$). An e-foil is steered by the rider dynamically shifting their center of gravity across pitch and roll axes, balanced against hydrodynamic lift from underwater wings, hull buoyancy, and motor thrust.
2. **Multi-Tiered Game Modes**: Broad appeal ranging from relaxing **Free Ride Exploration** to intense **Gate Slalom Racing**, **Style & Tricks Scoring**, **Swell Pumping** (motor-off wave riding), and **Battery Efficiency Challenges**.
3. **World Riding Destinations**: Varied riding environments with distinct water physics, landscapes, currents, and atmosphere: Corsica Beach, Norway Fjord, Arctic Sea, Garda Lake, Raglan Beach, and the Seine River in Paris.
4. **Modern SynthWave Electro Ambient Soundtrack**: Dynamic retro-futuristic music that organically morphs and intensifies as the rider transitions from displacement to high-speed foiling flight and deep carve turns.
5. **Plume Brand Showcase & Real-World Efoil Rewards**: Authentic representation of Plume's catalog of boards, wings, masts, and motors. A disruptive real-world reward program awards **2 to 5 real Plume efoils per year** to top seasonal champions.
6. **Cheat-Proof Competitive Integrity**: Deterministic physics with headless server-side input-log validation to ensure unhackable leaderboards and protect physical efoil prize integrity.
7. **Cross-Platform 60 FPS Scalability**: Seamless experience from high-end PC down to mobile smartphones, powered by lightweight custom shaders, procedural techniques, and mobile-first architecture.

---

## 2. Technical Architecture & Tech Stack

### 2.1 Engine & Core Frameworks
- **Language**: Rust (2024 edition). Memory safety, zero-cost abstractions, deterministic execution, and ultra-low CPU overhead.
- **Engine**: [Bevy Engine](https://bevyengine.org/) (0.15+).
  - ECS architecture for high cache efficiency and modular game systems.
  - Native cross-compilation for Windows, macOS, Linux, WebAssembly (WebGPU/WebGL2), Android, and iOS.
- **Physics**: `avian3d` (or `bevy_rapier3d`).
  - Rigid body dynamics running in deterministic `FixedUpdate` (60–120 Hz).
  - Custom accumulators for hydrofoil lift/drag, hull buoyancy, wave interaction, motor propulsion, and rider torque.
- **Input System**: `leafwing-input-manager`.
  - Unified input mapping across Keyboard/Mouse, Gamepad, and Mobile Touch Gestures.
  - Deterministic input logging (`InputLog`) recording exact frame-by-frame inputs for replay verification.
- **Water & Shaders**:
  - Analytical Gerstner wave displacement shared identically between Rust CPU (for buoyant queries) and WGSL shaders (for GPU rendering).
  - Low-overhead mobile PBR water: depth color absorption, foam crests, specular glints, and wake trails.
- **Audio**: `bevy_kira_audio` or Bevy native audio.
  - Spatial 3D sound effects (motor hum, foil singing, water spray, hull slap).
  - Multi-track SynthWave electro ambient music system with adaptive stems.
- **Anti-Cheat & Validation**:
  - Headless verification binary (`validator_cli`) executing player input logs on a deterministic server to certify times/scores before leaderboard publishing.
- **Asset Pipeline**:
  - Automated conversion pipeline: Plume engineering CAD (STEP) $\to$ optimized glTF 2.0 / GLB meshes with LODs and simplified collision hulls.

---

## 3. Physics & Hydrodynamics Specification

An efoil operating across water experiences distinct operational regimes:

```
[Displacement Phase] ---> [Planing Transition] ---> [Foiling Flight] ---> [High-Speed Carving]
(Hull floats; high drag)   (Lift exceeds weight;     (Hull airborne;      (Banked turns, induced
                            hull leaves water)        minimal drag)        drag, wake interaction)
                                      |
                                      v
                             [Surface Breach / Stall]
                             (Wing pierces surface;
                              sudden loss of lift)
```

### 3.1 Force Equations & Component Physics
At each fixed physics step $\Delta t$, the simulation evaluates:

1. **Buoyancy (Archimedes)**:
   - Sampled across an array of 4–8 probe points along the board hull.
   - For each probe submerged by depth $d_i = \max(0, h_{\text{wave}}(x_i, z_i) - y_i)$:
     $$\vec{F}_{\text{buoyant}, i} = \rho_{\text{water}} \cdot V_i(d_i) \cdot \vec{g}$$
   - Water density $\rho_{\text{water}}$ adapts to location (e.g., $1025\,\text{kg/m}^3$ ocean vs. $997\,\text{kg/m}^3$ freshwater lake).

2. **Hydrofoil Lift & Drag (Front Wing & Stabilizer)**:
   - Evaluated at wing hydrodynamic centers.
   - Local relative flow velocity $\vec{v}_{\text{rel}} = \vec{v}_{\text{wing}} - \vec{v}_{\text{water\_current}}$ (supporting directional river currents in Paris).
   - Angle of attack $\alpha = \angle(\text{chord\_axis}, \vec{v}_{\text{rel}})$.
   - Lift force:
     $$L = \frac{1}{2} \rho_{\text{water}} |\vec{v}_{\text{rel}}|^2 S C_L(\alpha)$$
   - Drag force (Profile + Induced):
     $$D = \frac{1}{2} \rho_{\text{water}} |\vec{v}_{\text{rel}}|^2 S \left( C_{D0} + \frac{C_L^2}{\pi \cdot AR \cdot e} \right)$$
   - **Stall & Breach Dynamics**: If $\alpha > \alpha_{\text{stall}}$ or if the wing breaks the water surface ($y_{\text{wing}} \ge h_{\text{wave}}$), $C_L \to 0$ to simulate air ventilation/aeration, causing realistic nose drop or wipeout.

3. **Propulsion & Motor**:
   - Thrust aligned with fuselage motor axis:
     $$T = k_T \cdot \text{throttle} \cdot \left(1 - \frac{v_{\text{advance}}}{v_{\text{max\_prop}}}\right)$$
   - Battery state: Voltage curve discharge model based on throttle draw and motor load.

4. **Rider Counter-Balancing Dynamics (Unstable Equilibrium, Gravity & Centrifugal Force)**:
   - Hydrofoil flight operates in a regime of **inherently unstable equilibrium**. Hydrodynamic lift acting at the underwater wing's center of pressure generates strong pitching moments that vary nonlinearly with angle of attack ($\alpha$) and velocity.
   - Stable flight is maintained dynamically by the rider acting as an active **counter-balancing force and torque vector** applied to the board:
     $$\vec{F}_{\text{rider}} = \vec{F}_{\text{rider, gravity}} + \vec{F}_{\text{rider, centrifugal}}$$
     where:
     - **Gravity Force**: $\vec{F}_{\text{rider, gravity}} = m_{\text{rider}} \vec{g}$ (downward in world space).
     - **Centrifugal Force (Carving)**: When carving turns at angular velocity $\vec{\omega}$ and linear velocity $\vec{v}$:
       $$\vec{F}_{\text{rider, centrifugal}} = -m_{\text{rider}} (\vec{\omega} \times \vec{v})$$
       When banking, the rider leans inward to balance centrifugal force against gravity in a coordinated turn.
   - **Net Counter-Balancing Torque**: Applied to the efoil rigid body about its center of mass:
     $$\vec{\tau}_{\text{rider}} = \vec{r}_{\text{rider}} \times \vec{F}_{\text{rider}}$$
     - **Pitch Axis**: Fore/aft weight shift controls $\vec{r}_{\text{rider}} \cdot \hat{z}$, balancing wing pitching moments and stabilizer downforce to hold the narrow angle-of-attack window for level flight.
     - **Roll Axis**: Heel/toe lateral shift controls $\vec{r}_{\text{rider}} \cdot \hat{x}$, initiating banking and balancing against centrifugal carve forces.
   - **Flight Envelope Stability**: If the rider's counter-balancing falls outside the operating window, positive feedback takes over (over-pitching $\to$ foil surface breach/stall; under-pitching $\to$ nosedive/pearl), creating authentic "easy to learn, hard to master" flight dynamics.

---

## 4. World Riding Destinations & Waterways

Each environment features distinct visual aesthetics, wave mechanics, water density, and gameplay challenges:

| Environment | Geography & Vibe | Water & Wave Characteristics | Unique Gameplay Challenge |
|---|---|---|---|
| **Corsica Beach** | Mediterranean sun, turquoise water, red granite cliffs | Mild swells, crystal clear, low chop ($\rho = 1025\,\text{kg/m}^3$) | Beginner friendly, wide gate slalom, scenic free cruising |
| **Norway Fjord** | Sheer towering cliffs, deep sheltered fjord, crisp air | Ultra-flat, mirror-like glassy water | Pure speed runs, precision carving, zero wave distraction |
| **Arctic Sea** | Drifting ice floes, glaciers, glowing Aurora Borealis | Frigid dark ocean, freezing sea spray, floating ice hazards | Extreme obstacle avoidance, technical precision slalom |
| **Garda Lake** | Italian alpine lake, steep mountain thermals, historic castles | Freshwater ($\rho = 997\,\text{kg/m}^3$), short choppy wind waves | Higher takeoff speed needed, choppy water hull-slap management |
| **Raglan Beach** | New Zealand black sand, lush headlands, world-class surf | Massive rolling point-break swells | Swell pumping (riding without motor), wave surfing on foil |
| **Seine River (Paris)** | Historic stone bridges, Eiffel Tower, urban quays | Directional current ($\vec{v} = 1.0\text{--}2.5\,\text{m/s}$), boat wake | Low bridge arch navigation, upstream vs. downstream tactics |

---

## 5. Game Modes & Progression

1. **Gate Slalom / Circuit Racing**:
   - Official competitive racing against the clock.
   - Floating buoy gates (red = pass right, green = pass left, yellow = speed trap).
   - Sector splits, time penalties for missed buoys, personal best ghost replays.
2. **Free Ride & Exploration**:
   - Zero-stress open exploration of large waterway environments.
   - Relaxing SynthWave soundtrack, scenic vistas, hidden coves, sunset/night cruises.
   - Battery management mode: discover how far you can cruise on a single charge.
3. **Style & Tricks Scoring**:
   - Judged runs scoring radical carving maneuvers:
     - High-G carve angle (measuring lean angle and lateral Gs).
     - Wingtip surface slice (carving so hard the upper wingtip slices the surface without breaching the main foil).
     - 180 / 360 surface flat spins.
     - Wave jumps and smooth re-entries.
4. **Swell Pumping Challenge (Zero-Motor)**:
   - Motor throttle disabled or cut after launch.
   - Players use rider leg rhythm and weight transfer to "pump" the foil, harvesting energy from ocean swells and maintaining endless flight.
5. **Battery Efficiency & Endurance**:
   - Maximize distance traveled on a limited battery capacity by discovering optimal foil trim and cruise throttle.
6. **Competitive Seasons (Plume Grand Prix)**:
   - Ranked monthly/seasonal ladders on standardized courses.
   - Real Plume efoils awarded to verified season champions.

---

## 6. Real-World Plume Efoil Reward & Anti-Cheat Architecture

Rewarding real physical Plume efoils (worth several thousand euros) creates massive player excitement, but requires **uncompromising anti-cheat security**.

```
[Game Client]                          [Backend Server]
      |                                        |
      | 1. Record deterministic InputLog       |
      |    (seeds, timestamped inputs)         |
      |                                        |
      | 2. Submit signed run payload --------> |
      |    (PlayerID, InputLog, Hash)          | 3. Headless Verification Runner:
      |                                        |    - Re-executes physics at FixedUpdate
      |                                        |    - Validates checkpoints & final time
      |                                        |    - Checks input frequency / bot heuristics
      |                                        |    - Validates physics bounds (Gs, speeds)
      |                                        |
      | 4. Verified & Certified <------------- | 5. Publish to Official Leaderboard
      |    Ghost replay published for peer     |    and assign Season Championship points
      |    review by community                 |
```

### Security Pillars
1. **Deterministic Input Replay Verification**:
   - The client **never** submits a raw time or score.
   - The client submits an authenticated **Input Stream** containing the seed and exact per-tick input events (throttle, pitch lean, roll lean).
   - A headless server-side runner (`validator_cli`) re-simulates the entire run tick-by-tick. If the simulated outcome diverges by even a fraction of a millisecond, the run is rejected.
2. **Bot & Macro Anomaly Detection**:
   - Algorithmic analysis of input telemetry: detection of superhuman step-function inputs ($0 \to 100\%$ instantaneous inputs), robotic frequency patterns, or impossible micro-corrections.
3. **Physical Plausibility Constraints**:
   - Hard physics assertion envelopes: maximum achievable foil acceleration, wing lift limits, and battery energy conservation checks.
4. **Community Ghost Replays (Peer Review)**:
   - The top 20 runs on every leaderboard are automatically downloadable as interactive ghost racers. The community can inspect, race against, and peer-review top runs.
5. **Verified Player Identification**:
   - Players qualifying for physical efoil reward tiers must undergo account verification (anti-sybil checks, KYC) to prevent multi-accounting.

---

## 7. Dynamic SynthWave Soundtrack & Sound Design

### 7.1 Musical Aesthetic
- **Genre**: Modern SynthWave / ChillSynth / Electro Ambient.
- **Vibe**: 80s retro-futuristic nostalgia meets modern electronic production—warm analog synthesizers, lush chorus pads, arpeggiated basslines, and dreamy reverberant soundscapes.
- **Dynamic Layering (Stem System)**:
  - *Layer 1 (Ambient Chill)*: Gentle synth pads, ocean ambiance, soft pulse while floating/cruising in displacement mode.
  - *Layer 2 (Flight Groove)*: Warm bassline and crisp drum beat fade in seamlessly as the efoil lifts off and accelerates.
  - *Layer 3 (High-Speed Arp)*: Pulsing synth arpeggiators intensify at top speeds ($> 35\,\text{km/h}$).
  - *Layer 4 (Carve Filter)*: Low-pass / high-pass filter sweeps dynamically modulate with rider roll angle and deep carves.
  - *Layer 5 (Breach/Crash Strum)*: Immediate filter drop and tape-stop effect when the foil breaches or crashes.

### 7.2 Hydrodynamic Sound FX
- **Electric Motor Whine**: High-frequency electric motor hum modulated dynamically by motor RPM and electrical load.
- **Hydrofoil Whistle / Singing**: Characteristic whistling tone of the carbon hydrofoil slicing through water, shifting pitch with speed.
- **Hull Slap & Spray**: Crisp water slap when the board touches down; rushing water spray based on submersion depth.

---

## 8. Phase-by-Phase Master Roadmap

### Phase 0: Project Setup, Toolchain & Architecture Scaffolding
- Initialize Rust workspace with Bevy 0.15+, `avian3d` (or `bevy_rapier3d`), and `leafwing-input-manager`.
- Configure Git repository: [Ka10U/plumefoil-racing](https://github.com/Ka10U/plumefoil-racing) with `.gitignore` and CI actions.
- Establish cross-compilation pipeline (Windows, macOS, Linux, Android, iOS, WASM).
- Configure fast linking (`lld`/`mold`, dynamic linking).

### Phase 1: Core Physics & Hydrodynamics Prototype
- Implement deterministic fixed-timestep hydrodynamics loop (`FixedUpdate`).
- Build multi-point hull buoyancy sampling system.
- Implement wing lift/drag equations ($C_L, C_D$), stall behavior, and surface breach ventilation.
- Implement motor thrust curve and hull hydrodynamic water drag.
- Create debug visualization tools (gizmos for forces, CoM, and buoyancy probes).

### Phase 2: Rider Dynamics, Input Abstraction & Camera
- Implement unified input handling using `leafwing-input-manager` (Keyboard, Gamepad, Touch).
- Develop rider dynamic stance model: weight shifting, pitch/roll balance, and lean torque.
- Build reactive follow camera system: smooth third-person chase camera with dynamic FOV, banking roll tilt, and ride-height tracking.
- Build basic wipeout/recovery state machine (pearling/nosedive, foil breach, over-banking).

### Phase 3: Water Simulation, Waves & Environment
- Develop analytical Gerstner wave mathematics library shared between CPU and GPU.
- Implement custom WGSL water shader featuring vertex displacement, depth color absorption, specular sun glint, and edge foam.
- Add support for variable water density ($\rho$) and river current vector fields.
- Build initial environment presets (Corsica Beach, Norway Fjord, Garda Lake, etc.).

### Phase 4: CAD Asset Pipeline & 3D Model Integration
- Develop automated conversion pipeline: Plume STEP files $\to$ triangulated meshes $\to$ optimized glTF/GLB.
- Generate level-of-detail (LOD) hierarchies and low-poly convex collision hulls.
- Create 3D rider model with procedural posture responsive to stance lean and crouch.
- Implement modular efoil assembly system (swap Plume boards, masts, front wings, stabilizers, and motors).

### Phase 5: Game Modes, Course System & Telemetry HUD
- Implement racing course system: floating buoy gates, slalom paths, directional arrows.
- Implement diverse game modes: Gate Slalom, Free Ride, Style & Tricks, Swell Pumping, Battery Endurance.
- Create telemetry HUD: speed (knots and km/h), altitude above water, battery charge, throttle percentage, pitch/roll inclinometer.

### Phase 6: Dynamic SynthWave Audio & Sound Design
- Integrate SynthWave electro ambient music engine with interactive stems.
- Implement dynamic sound synthesis for electric motor whine tied to motor RPM and load.
- Add hydrodynamic foil singing, water hull lapping, hull slap impacts, and coastal wind rush.

### Phase 7: Anti-Cheat Engine, Replay System & Leaderboards
- Build deterministic input recorder (`InputLog`) and serializer.
- Implement standalone headless physics validation runner (`validator_cli`).
- Add anomaly/bot detection heuristics.
- Build ghost replay recorder, playback engine, and verified leaderboard system.

### Phase 8: Mobile Optimization, Touch Controls & Packaging
- Design ergonomic mobile UI: left thumb virtual analog stick (stance lean), right thumb throttle slider/lever.
- Optimize rendering pipeline for mobile GPUs (Vulkan/Metal/OpenGLES): shader simplification, texture compression (KTX2/Basis), draw call batching.
- Package Android APK and iOS build targets.

### Phase 9: Plume Garage, Brand Showcase & Reward System
- Create "Plume Garage" showroom: 3D interactive viewer to inspect, configure, and compare real Plume efoil specs.
- Build the Plume Grand Prix reward hub and seasonal leaderboard interface.
- Final UI/UX polish, transitions, accessibility settings, and sound balance.

---

## 9. Granular Implementation Plans Index

The master implementation is decomposed into dedicated, standalone implementation plans stored in the `Implementation Plans/` directory:

| # | Plan File | Focus Area | Key Deliverables |
|---|---|---|---|
| **00** | `00_project_setup_and_toolchain.md` | Scaffolding & CI/CD | Cargo workspace, Bevy boilerplate, fast linkers, GitHub Actions |
| **01** | `01_hydrodynamic_physics_and_buoyancy.md` | Core Flight Simulation | Buoyancy probe array, thin-airfoil lift/drag, stall, ventilation, gizmos |
| **02** | `02_efoil_assembly_and_component_model.md` | Component Data Model | Board, mast, wing, battery, motor ECS entities & data-driven specs |
| **03** | `03_rider_stance_and_control_mechanics.md` | Stance & Input | Weight-shift torque, input action maps (gamepad/keyboard), wipeout states |
| **04** | `04_water_shader_and_wave_synchronization.md` | Water & Shaders | Analytical Gerstner CPU math, WGSL mobile water material, foam & depth |
| **05** | `05_environments_and_waterways.md` | World Riding Areas | Corsica, Norway, Arctic, Garda, Raglan, Seine (currents & obstacles) |
| **06** | `06_cad_step_to_gltf_asset_pipeline.md` | CAD & Asset Pipeline | Python/FreeCAD/Blender STEP-to-glTF conversion, LODs, collision hulls |
| **07** | `07_game_modes_and_course_system.md` | Game Modes & Racing | Slalom racing, free ride, tricks score, swell pumping, buoy gates |
| **08** | `08_synthwave_audio_and_sound_design.md` | Music & Sound FX | Interactive SynthWave stem player, motor RPM whine, foil whistling, slap |
| **09** | `09_anti_cheat_and_replay_verification.md` | Competitive Security | Deterministic InputLog, headless validator CLI, ghost replay, anti-bot |
| **10** | `10_ui_hud_telemetry_and_mobile_controls.md` | UI & On-Screen Controls | Cockpit telemetry HUD, mobile virtual stick & throttle, settings UI |
| **11** | `11_mobile_optimization_and_cross_platform.md` | Mobile Optimization | Android/iOS build scripts, KTX2 textures, frame pacing, LOD system |
| **12** | `12_plume_garage_and_reward_system.md` | Plume Garage & Efoil Prize | 3D efoil configurator, Plume Grand Prix reward portal, final polish |

---

## 10. Technical Risks & Mitigation Strategies

| Risk | Impact | Likelihood | Mitigation Strategy |
|---|---|---|---|
| **Leaderboard Tampering / Hackers** | Critical (financial/brand loss via physical efoil rewards) | High | Mandatory deterministic headless server replay validation. No client score submissions accepted without verifiable input stream. |
| **Non-Deterministic Physics Across Platforms** | High (replay divergence between PC and server) | Medium | Strict use of fixed-timestep physics (`FixedUpdate`), seeded PRNGs, avoiding floating-point architecture drift, and running verification against standardized headless runner. |
| **Water CPU/GPU Discrepancy** | High (visual vs physical height mismatch) | Medium | Identical analytical Gerstner wave formulation in CPU Rust and GPU WGSL using synchronized game time. |
| **Mobile Thermal Throttling & Frame Drops** | High (unplayable touch controls) | Medium | Mobile-first shader design, geometry decimation ($< 25\text{k}$ tris for efoil), dynamic resolution scaling, and 60 FPS cap. |
| **CAD (STEP) Complexity** | Medium (excessive polygon density) | High | Automated asset pipeline decimation and convex hull generation for physics colliders. |

---

## 11. Immediate Next Steps

1. Commit and push current workspace to the newly created GitHub repository `Ka10U/plumefoil-racing`.
2. Generate implementation plan `Implementation Plans/00_project_setup_and_toolchain.md` to establish the Rust/Bevy workspace, toolchain, and CI/CD pipelines.
3. Proceed to `Implementation Plans/01_hydrodynamic_physics_and_buoyancy.md` for core physics development.
