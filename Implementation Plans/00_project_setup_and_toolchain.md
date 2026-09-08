# Implementation Plan 00: Project Setup, Toolchain & Scaffolding

> **Parent Document**: [Master Implementation Plan](file:///c:/05%20Vibe/Plumefoil%20Racing/Master_Implementation_Plan.md)  
> **Target Engine**: Rust + Bevy 0.15+  
> **Status**: Ready for Execution  

---

## 1. Overview & Objectives

The goal of **Implementation Plan 00** is to establish a robust, production-grade foundation for **Plumefoil Racing**. This plan sets up the multi-crate Cargo workspace, developer ergonomics (fast linking and dev compilation profiles), core dependencies, cross-platform build readiness, GitHub Actions CI, and a runnable minimalist 3D Bevy test scene with diagnostic telemetry.

### Key Objectives
1. **Modular Workspace Architecture**: Separate the game client, core hydrodynamics simulation, and headless validator CLI into distinct crates to enable zero-overhead server verification and rapid local testing.
2. **Fast Development Loop**: Configure `.cargo/config.toml` with `rust-lld` on Windows and optimized dev profiles (`opt-level = 1` for local code, `opt-level = 3` for dependencies) so debug builds achieve 60 FPS without slow compilation times.
3. **Core Dependencies Setup**: Integrate Bevy 0.15, `avian3d` (or `bevy_rapier3d`), `leafwing-input-manager`, and serialization tools (`serde`, `bincode`).
4. **Asset Directory Hierarchy**: Set up standard directories for models, shaders, textures, audio, and music.
5. **Continuous Integration**: Configure GitHub Actions to automatically run `cargo fmt`, `cargo clippy`, and `cargo test` on every push to `main`.
6. **Minimal Viable 3D App**: Build and run an initial 3D scene verifying windowing, 3D camera, diagnostic telemetry (FPS counter), and clean shutdown.

---

## 2. Workspace Structure & Crate Layout

A multi-crate Cargo workspace isolates headless deterministic logic from GPU/windowing dependencies:

```
plumefoil-racing/
├── .cargo/
│   └── config.toml                  # rust-lld linker, dynamic linking flags
├── .github/
│   └── workflows/
│       └── ci.yml                   # Automated fmt, clippy, and test pipeline
├── assets/
│   ├── models/                      # glTF/GLB models & collision hulls
│   ├── shaders/                     # WGSL shaders (water, sky, post-fx)
│   ├── textures/                    # PBR textures
│   ├── music/                       # SynthWave electro ambient tracks & stems
│   └── audio/                       # Sound effects (motor, hydrofoil whistling, slap)
├── crates/
│   ├── plumefoil_core/              # Common math, units, physical constants, replay types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── constants.rs         # Water densities, gravity, standard atmospheric values
│   │       └── replay.rs            # Deterministic InputLog and telemetry packet schemas
│   ├── plumefoil_physics/           # Deterministic hydrodynamics & buoyancy simulation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── lift_drag.rs         # Foil polars, NACA/custom curves, stall formulas
│   │       ├── buoyancy.rs          # Multi-probe hull immersion math
│   │       └── thrust.rs            # Electric motor & propeller torque curves
│   └── plumefoil_validator/         # Standalone headless replay verification CLI
│       ├── Cargo.toml
│       └── src/
│           └── main.rs              # Re-simulates input logs without GPU/window dependencies
├── src/                             # Main game binary (Bevy application)
│   ├── main.rs                      # Entry point, plugin registration, window setup
│   └── diagnostics.rs               # Frame-rate and system performance overlay
├── Cargo.toml                       # Workspace manifest
└── README.md
```

### Rationale
- **`plumefoil_core` & `plumefoil_physics`**: Free of windowing and audio dependencies. Can be tested in microseconds on any platform.
- **`plumefoil_validator`**: Lightweight server CLI for anti-cheat verification. Can run on ultra-low-cost headless Linux servers without X11 or Vulkan.
- **Root binary (`src/main.rs`)**: High-performance Bevy game client consuming the above crates plus audio, rendering, input, and UI plugins.

---

## 3. Dependency Specification

### Workspace Root `Cargo.toml`
```toml
[workspace]
members = [
    ".",
    "crates/plumefoil_core",
    "crates/plumefoil_physics",
    "crates/plumefoil_validator",
]
resolver = "2"

[workspace.dependencies]
bevy = { version = "0.15", default-features = true }
avian3d = { version = "0.2", default-features = true }
leafwing-input-manager = "0.16"
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
glam = "0.29"
thiserror = "2.0"
tracing = "0.1"

# Internal crates
plumefoil_core = { path = "crates/plumefoil_core" }
plumefoil_physics = { path = "crates/plumefoil_physics" }
```

### Dev vs. Release Profiles
```toml
# Fast dev compile times with smooth runtime performance
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

# Optimized release profile for physics profiling and production builds
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
```

---

## 4. Development Configuration (`.cargo/config.toml`)

```toml
[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"

# Enable dynamic linking during rapid development iteration
# Uncomment when iterating quickly on game logic:
# [dependencies]
# bevy = { version = "0.15", features = ["dynamic_linking"] }
```

---

## 5. Step-by-Step Execution Plan

### Step 1: Initialize Workspace Manifest & Directory Tree
- Create root `Cargo.toml` with workspace members and shared dependency specifications.
- Create `.cargo/config.toml` configuring the fast `rust-lld` linker.
- Create folder tree for `assets/` (`models`, `shaders`, `textures`, `music`, `audio`).

### Step 2: Implement `plumefoil_core` Crate
- Set up `crates/plumefoil_core/Cargo.toml`.
- Implement `constants.rs`:
  - Seawater density ($\rho_{\text{sea}} = 1025.0\,\text{kg/m}^3$)
  - Freshwater density ($\rho_{\text{fresh}} = 997.0\,\text{kg/m}^3$)
  - Gravity ($g = 9.80665\,\text{m/s}^2$)
- Implement `replay.rs`:
  - `InputFrame` struct (timestamp, throttle, pitch_lean, roll_lean)
  - `InputLog` struct (session seed, player_id, course_id, frames) with serde serialization.
- Add unit tests verifying serialization and deserialization.

### Step 3: Implement `plumefoil_physics` Crate Scaffold
- Set up `crates/plumefoil_physics/Cargo.toml` referencing `plumefoil_core`.
- Stub initial modules: `lift_drag.rs`, `buoyancy.rs`, `thrust.rs`.
- Add smoke test verifying hydrodynamics calculations run headlessly.

### Step 4: Implement `plumefoil_validator` CLI Scaffold
- Set up `crates/plumefoil_validator/Cargo.toml`.
- Implement `main.rs` accepting a `.pfr` (Plumefoil Replay) file path argument and printing validation status.

### Step 5: Implement Main Game Client Scaffold (`src/main.rs`)
- Configure main application with Bevy 0.15 default plugins.
- Set window properties: title `"Plumefoil Racing"`, resolution $1280 \times 720$, vsync enabled.
- Spawn a 3D camera, directional sunlight, and a placeholder floating grid/water plane.
- Add an FPS / diagnostic text overlay to verify performance.

### Step 6: Configure GitHub Actions CI (`.github/workflows/ci.yml`)
- Automate multi-platform checks on push and pull requests:
  - Rust format check (`cargo fmt --check`).
  - Clippy linter (`cargo clippy --workspace --all-targets -- -D warnings`).
  - Unit tests (`cargo test --workspace`).

---

## 6. Verification Protocol

| Check | Command | Expected Result |
|---|---|---|
| **Compilation** | `cargo check --workspace` | Completes with 0 errors and 0 warnings |
| **Formatting** | `cargo fmt --check` | Clean formatting across all crates |
| **Linting** | `cargo clippy --workspace` | Clean clippy check with no warnings |
| **Unit Tests** | `cargo test --workspace` | All tests pass in `plumefoil_core` and `plumefoil_physics` |
| **Validator CLI** | `cargo run --bin plumefoil_validator -- --help` | Displays usage and exit code 0 |
| **Interactive App** | `cargo run` | Opens 3D window, renders scene, displays FPS counter at 60 FPS, exits cleanly |

---

## 7. Status & Next Milestones

Upon completion and verification of this plan, the repository will be committed and pushed to `Ka10U/plumefoil-racing`, and work will begin on **Implementation Plan 01: Hydrodynamic Physics & Buoyancy Prototype**.
