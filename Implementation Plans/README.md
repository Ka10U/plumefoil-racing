# Implementation Plans

This directory contains granular, execution-ready implementation plans for **Plumefoil Racing**, broken down from the [Master Implementation Plan](file:///c:/05%20Vibe/Plumefoil%20Racing/Master_Implementation_Plan.md).

## Sub-Implementation Plans Index

| # | Plan File | Focus Area | Status |
|---|---|---|---|
| **00** | `00_project_setup_and_toolchain.md` | Cargo workspace, Bevy 0.15+, fast linkers, GitHub Actions CI/CD | Completed |
| **01** | `01_hydrodynamic_physics_and_buoyancy.md` | Multi-point buoyancy, wing lift/drag, stall, ventilation, force gizmos | Completed |
| **02** | `02_efoil_assembly_and_component_model.md` | Modular efoil data model (board, mast, wings, motor, battery) | Pending |
| **03** | `03_rider_stance_and_control_mechanics.md` | Rider dynamic center of mass, weight shift torque, input abstraction | Pending |
| **04** | `04_water_shader_and_wave_synchronization.md` | Analytical Gerstner CPU math, WGSL mobile water material, foam & depth | Pending |
| **05** | `05_environments_and_waterways.md` | World riding areas: Corsica, Norway, Arctic, Garda, Raglan, Seine currents | Pending |
| **06** | `06_cad_step_to_gltf_asset_pipeline.md` | Automated STEP to glTF/GLB conversion, LODs, collision hulls | Pending |
| **07** | `07_game_modes_and_course_system.md` | Gate slalom racing, free ride, style/tricks scoring, swell pumping | Pending |
| **08** | `08_synthwave_audio_and_sound_design.md` | Dynamic SynthWave electro ambient stem player, motor RPM whine, foil whistling | Pending |
| **09** | `09_anti_cheat_and_replay_verification.md` | Deterministic InputLog, headless validator CLI, ghost replays, bot detection | Pending |
| **10** | `10_ui_hud_telemetry_and_mobile_controls.md` | Telemetry HUD, mobile virtual stick & throttle slider, settings UI | Pending |
| **11** | `11_mobile_optimization_and_cross_platform.md` | Mobile GPU optimizations, Android APK / iOS builds, KTX2 compression | Pending |
| **12** | `12_plume_garage_and_reward_system.md` | 3D Plume Garage configurator, real-world efoil reward portal, final polish | Pending |

---

*Refer to [GEMINI.md](file:///c:/05%20Vibe/Plumefoil%20Racing/GEMINI.md) for coding standards, architectural rules, and engine conventions.*
