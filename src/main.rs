//! Plumefoil Racing - Main Game Client
//! Built with Rust, Bevy Engine 0.15, and Avian3D 0.2.

mod efoil_components;
mod flight_debug;
mod physics_plugin;

use avian3d::prelude::*;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use efoil_components::*;
use flight_debug::FlightDebugPlugin;
use physics_plugin::EfoilPhysicsPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Plumefoil Racing - Physics & Hydrodynamics Simulation".into(),
                        resolution: (1280.0, 720.0).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                }),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(EfoilPhysicsPlugin)
        .add_plugins(FlightDebugPlugin)
        .insert_resource(ClearColor(Color::srgb(0.06, 0.10, 0.16)))
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                handle_flight_input_system,
                update_camera_follow_system,
                update_hud_telemetry_system,
            ),
        )
        .run();
}

/// Marker component for third-person chase camera.
#[derive(Component)]
struct FollowCamera {
    pub offset: Vec3,
    pub smooth_speed: f32,
}

impl Default for FollowCamera {
    fn default() -> Self {
        Self {
            offset: Vec3::new(0.0, 2.2, -6.0),
            smooth_speed: 6.0,
        }
    }
}

/// Marker component for HUD telemetry readout.
#[derive(Component)]
struct HudTelemetryText;

/// Marker component for HUD flight state readout.
#[derive(Component)]
struct HudStateText;

/// Marker component for FPS diagnostic text.
#[derive(Component)]
struct FpsText;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 3D Perspective Chase Camera
    commands.spawn((
        Camera3d::default(),
        FollowCamera::default(),
        Transform::from_xyz(0.0, 3.0, -7.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));

    // Directional Sunlight (Warm Mediterranean Sun)
    commands.spawn((
        DirectionalLight {
            illuminance: 16_000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.97, 0.92),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.65, 0.6, 0.0)),
    ));

    // Ambient Light Fill
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.65, 0.78, 0.92),
        brightness: 350.0,
    });

    // Stylized Water Surface Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(250.0, 250.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.02, 0.16, 0.26),
            metallic: 0.15,
            perceptual_roughness: 0.12,
            reflectance: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Visual Materials for Plume Efoil
    let board_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.94, 0.95, 0.97), // Plume gloss white
        metallic: 0.25,
        perceptual_roughness: 0.25,
        ..default()
    });

    let carbon_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.10, 0.12), // Carbon black
        metallic: 0.6,
        perceptual_roughness: 0.35,
        ..default()
    });

    let accent_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.95), // Plume electric cyan
        metallic: 0.4,
        perceptual_roughness: 0.3,
        ..default()
    });

    // Compound Physical Efoil Entity
    let board_shape = Collider::cuboid(0.68, 0.12, 1.60);
    let mast_shape = Collider::cuboid(0.04, 0.85, 0.15);
    let wing_shape = Collider::cuboid(0.92, 0.02, 0.18);
    let stab_shape = Collider::cuboid(0.38, 0.015, 0.09);

    let compound_collider = Collider::compound(vec![
        (Vec3::new(0.0, 0.0, 0.0), Quat::IDENTITY, board_shape),
        (Vec3::new(0.0, -0.45, -0.20), Quat::IDENTITY, mast_shape),
        (Vec3::new(0.0, -0.85, 0.15), Quat::IDENTITY, wing_shape),
        (Vec3::new(0.0, -0.84, -0.65), Quat::IDENTITY, stab_shape),
    ]);

    commands
        .spawn((
            EfoilRigidBody,
            RigidBody::Dynamic,
            compound_collider,
            Mass(110.0), // 35 kg hardware + 75 kg rider
            ExternalForce::default(),
            ExternalTorque::default(),
            LinearVelocity::default(),
            AngularVelocity::default(),
            Transform::from_xyz(0.0, 0.10, 0.0), // Start resting on water surface
            Visibility::default(),
            HullBuoyancyComponent::default(),
            HydrofoilWingsComponent::default(),
            EfoilPropulsionComponent::default(),
            RiderCounterBalance::default(),
            FlightTelemetry::default(),
        ))
        .with_children(|parent| {
            // Visual Board Mesh
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.68, 0.12, 1.60))),
                MeshMaterial3d(board_mat),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));

            // Visual Grip Pad / Deck Accent
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.50, 0.01, 1.10))),
                MeshMaterial3d(accent_mat),
                Transform::from_xyz(0.0, 0.065, -0.05),
            ));

            // Visual Carbon Mast
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.03, 0.85, 0.14))),
                MeshMaterial3d(carbon_mat.clone()),
                Transform::from_xyz(0.0, -0.45, -0.20),
            ));

            // Visual Front Hydrofoil Wing
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.92, 0.02, 0.18))),
                MeshMaterial3d(carbon_mat.clone()),
                Transform::from_xyz(0.0, -0.85, 0.15),
            ));

            // Visual Rear Stabilizer Wing
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.38, 0.015, 0.09))),
                MeshMaterial3d(carbon_mat),
                Transform::from_xyz(0.0, -0.84, -0.65),
            ));
        });

    // Telemetry & Diagnostic HUD Overlay
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("PLUMEFOIL RACING"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.85, 0.95)),
            ));

            parent.spawn((
                FpsText,
                Text::new("FPS: -- | Frame Time: -- ms"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.88, 0.92)),
            ));

            parent.spawn((
                HudTelemetryText,
                Text::new("Speed: 0.0 km/h (0.0 kts) | Alt: 0.10 m | Throttle: 0%"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.9, 0.7)),
            ));

            parent.spawn((
                HudStateText,
                Text::new("Flight Regime: [Floating] | G-Force: 1.0 G"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.7, 0.95)),
            ));

            parent.spawn((
                Text::new(
                    "Controls: [Space/Shift] Throttle | [W/S] Pitch Trim | [A/D] Carve Roll | [R] Reset | [F1] Force Gizmos",
                ),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.65, 0.75)),
            ));
        });
}

/// Reads keyboard input and updates throttle and rider stance pitch/roll.
fn handle_flight_input_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut EfoilPropulsionComponent,
            &mut RiderCounterBalance,
        ),
        With<EfoilRigidBody>,
    >,
) {
    let dt = time.delta_secs();

    for (mut transform, mut linear_vel, mut angular_vel, mut propulsion, mut rider) in &mut query {
        // Reset efoil to start position
        if keyboard.just_pressed(KeyCode::KeyR) {
            transform.translation = Vec3::new(0.0, 0.10, 0.0);
            transform.rotation = Quat::IDENTITY;
            linear_vel.0 = Vec3::ZERO;
            angular_vel.0 = Vec3::ZERO;
            propulsion.target_throttle = 0.0;
            propulsion.current_throttle = 0.0;
            rider.pitch_lean = 0.0;
            rider.roll_lean = 0.0;
            continue;
        }

        // 1. Throttle Input: Space = Accelerate, Shift = Decelerate
        if keyboard.pressed(KeyCode::Space) {
            propulsion.target_throttle = (propulsion.target_throttle + 0.45 * dt).min(1.0);
        } else if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            propulsion.target_throttle = (propulsion.target_throttle - 0.60 * dt).max(0.0);
        }

        // 2. Rider Pitch Lean (Fore / Aft Weight Shift)
        // W / Up = Lean forward (Nose Down / Dive)
        // S / Down = Lean back (Nose Up / Climb)
        let mut target_pitch = 0.0;
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            target_pitch -= 0.8;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            target_pitch += 0.8;
        }
        rider.pitch_lean += (target_pitch - rider.pitch_lean) * (8.0 * dt).min(1.0);

        // 3. Rider Roll Lean (Heel / Toe Carving Weight Shift)
        // A / Left = Lean Left (Bank Left)
        // D / Right = Lean Right (Bank Right)
        let mut target_roll = 0.0;
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            target_roll -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            target_roll += 1.0;
        }
        rider.roll_lean += (target_roll - rider.roll_lean) * (6.0 * dt).min(1.0);
    }
}

/// Smooth third-person chase camera following the physical efoil.
fn update_camera_follow_system(
    time: Res<Time>,
    efoil_query: Query<&Transform, (With<EfoilRigidBody>, Without<FollowCamera>)>,
    mut camera_query: Query<(&mut Transform, &FollowCamera)>,
) {
    let Ok(efoil_transform) = efoil_query.get_single() else {
        return;
    };
    let Ok((mut cam_transform, follow)) = camera_query.get_single_mut() else {
        return;
    };

    let efoil_pos = efoil_transform.translation;
    let forward = efoil_transform.forward().as_vec3();
    let up = Vec3::Y;

    // Camera target: behind and above the efoil along its heading
    let desired_pos = efoil_pos + forward * follow.offset.z + up * follow.offset.y;
    let dt = time.delta_secs();
    cam_transform.translation = cam_transform
        .translation
        .lerp(desired_pos, (follow.smooth_speed * dt).min(1.0));

    // Look slightly ahead of the efoil nose
    let look_target = efoil_pos + forward * 2.0 + up * 0.4;
    cam_transform.look_at(look_target, Vec3::Y);
}

/// Updates the HUD overlay with real-time telemetry and flight states.
#[allow(clippy::type_complexity)]
fn update_hud_telemetry_system(
    diagnostics: Res<DiagnosticsStore>,
    efoil_query: Query<
        (
            &FlightTelemetry,
            &EfoilPropulsionComponent,
            &RiderCounterBalance,
        ),
        With<EfoilRigidBody>,
    >,
    mut fps_query: Query<
        &mut Text,
        (
            With<FpsText>,
            Without<HudTelemetryText>,
            Without<HudStateText>,
        ),
    >,
    mut telemetry_query: Query<
        &mut Text,
        (
            With<HudTelemetryText>,
            Without<FpsText>,
            Without<HudStateText>,
        ),
    >,
    mut state_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<HudStateText>,
            Without<FpsText>,
            Without<HudTelemetryText>,
        ),
    >,
) {
    // 1. FPS / Frame Time
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed());
    let ft = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed());

    if let (Some(fps_val), Some(ft_val)) = (fps, ft) {
        for mut text in &mut fps_query {
            **text = format!("FPS: {fps_val:.1} | Frame Time: {ft_val:.2} ms");
        }
    }

    // 2. Flight Telemetry Readouts
    let Ok((telemetry, propulsion, rider)) = efoil_query.get_single() else {
        return;
    };

    for mut text in &mut telemetry_query {
        **text = format!(
            "Speed: {:.1} km/h ({:.1} kts) | Alt: {:.2} m | Wing: {:.2} m | Throttle: {:.0}%",
            telemetry.speed_kmh,
            telemetry.speed_knots,
            telemetry.ride_height,
            telemetry.wing_depth,
            propulsion.current_throttle * 100.0
        );
    }

    // 3. Flight State & Dynamic Coloring
    for (mut text, mut color) in &mut state_query {
        let (state_str, state_color) = match telemetry.state {
            FlightState::Floating => ("Floating", Color::srgb(0.3, 0.65, 0.95)),
            FlightState::Planing => ("Planing Transition", Color::srgb(0.95, 0.8, 0.2)),
            FlightState::Foiling => ("Foiling Flight", Color::srgb(0.2, 0.95, 0.5)),
            FlightState::Breached => ("VENTILATED / BREACHED", Color::srgb(0.95, 0.25, 0.2)),
        };

        **text = format!(
            "State: [{state_str}] | Pitch: {:.1}° | Roll: {:.1}° | Lean: [{:+.2}, {:+.2}]",
            telemetry.pitch_deg, telemetry.roll_deg, rider.pitch_lean, rider.roll_lean,
        );
        color.0 = state_color;
    }
}
