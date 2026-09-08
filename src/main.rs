//! Plumefoil Racing - Main Game Client
//! Built with Rust and Bevy Engine 0.15.

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use plumefoil_core::constants::{MPS_TO_KMH, MPS_TO_KNOTS};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Plumefoil Racing".into(),
                        resolution: (1280.0, 720.0).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // Watch for asset changes during development
                    watch_for_changes_override: Some(true),
                    ..default()
                }),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .insert_resource(ClearColor(Color::srgb(0.08, 0.12, 0.18)))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (update_fps_overlay, animate_water_bobbing))
        .run();
}

/// Marker component for the FPS diagnostic text.
#[derive(Component)]
struct FpsText;

/// Marker component for animated efoil demonstration entity.
#[derive(Component)]
struct DemoEfoil {
    base_y: f32,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 3D Perspective Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.5, -6.5).looking_at(Vec3::new(0.0, 0.6, 0.0), Vec3::Y),
    ));

    // Directional Sunlight
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.96, 0.90),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.65, 0.6, 0.0)),
    ));

    // Ambient light fill
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6, 0.75, 0.9),
        brightness: 300.0,
    });

    // Stylized Water Surface Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(120.0, 120.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.02, 0.18, 0.28),
            metallic: 0.1,
            perceptual_roughness: 0.15,
            reflectance: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Placeholder Efoil Entity (Board + Mast + Wing)
    let board_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.93, 0.95), // Clean Plume white finish
        metallic: 0.3,
        perceptual_roughness: 0.3,
        ..default()
    });

    let carbon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.12, 0.14), // Matte carbon black
        metallic: 0.5,
        perceptual_roughness: 0.4,
        ..default()
    });

    // Efoil parent entity
    commands
        .spawn((
            DemoEfoil { base_y: 0.75 },
            Transform::from_xyz(0.0, 0.75, 0.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Board Hull
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.65, 0.12, 1.6))),
                MeshMaterial3d(board_material),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));

            // Mast (connecting board to underwater foil)
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.03, 0.85, 0.14))),
                MeshMaterial3d(carbon_material.clone()),
                Transform::from_xyz(0.0, -0.45, -0.25),
            ));

            // Front Hydrofoil Wing
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.90, 0.02, 0.18))),
                MeshMaterial3d(carbon_material.clone()),
                Transform::from_xyz(0.0, -0.85, -0.15),
            ));

            // Rear Stabilizer Wing
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.38, 0.015, 0.09))),
                MeshMaterial3d(carbon_material),
                Transform::from_xyz(0.0, -0.84, -0.65),
            ));
        });

    // Telemetry & Diagnostics HUD Overlay
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(12.0)),
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

            let sample_speed_mps = 8.5;
            let sample_kmh = sample_speed_mps * MPS_TO_KMH;
            let sample_knots = sample_speed_mps * MPS_TO_KNOTS;

            parent.spawn((
                Text::new(format!(
                    "Telemetry: {sample_kmh:.1} km/h ({sample_knots:.1} kts) | Height: 0.45 m"
                )),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.9, 0.7)),
            ));

            parent.spawn((
                Text::new("Engine: Bevy 0.15 | Avian3D 0.2 | Status: Ready"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.65, 0.75)),
            ));
        });
}

/// Updates the FPS / Frame Time text readout.
fn update_fps_overlay(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed());
    let ft = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed());

    if let (Some(fps_val), Some(ft_val)) = (fps, ft) {
        for mut text in &mut query {
            **text = format!("FPS: {fps_val:.1} | Frame Time: {ft_val:.2} ms");
        }
    }
}

/// Gentle sinusoidal hovering animation representing foiling flight dynamics.
fn animate_water_bobbing(time: Res<Time>, mut query: Query<(&mut Transform, &DemoEfoil)>) {
    let t = time.elapsed_secs();
    for (mut transform, efoil) in &mut query {
        let hover_offset = (t * 2.0).sin() * 0.08;
        let pitch_tilt = (t * 1.5).cos() * 0.02;
        let roll_tilt = (t * 1.2).sin() * 0.035;

        transform.translation.y = efoil.base_y + hover_offset;
        transform.rotation = Quat::from_euler(EulerRot::XYZ, pitch_tilt, 0.0, roll_tilt);
    }
}
