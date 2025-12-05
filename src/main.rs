//! GaussRace - A racing game with Gaussian splatting
//!
//! This game allows the user to:
//! 1. Load a Gaussian splat file (.ply)
//! 2. Select a ground plane within the splat
//! 3. Drive a vehicle around on that plane

use bevy::prelude::*;
use bevy_gaussian_splatting::GaussianSplattingPlugin;

mod car;
mod ground_plane;
mod splat_loader;

use car::CarPlugin;
use ground_plane::GroundPlanePlugin;
use splat_loader::SplatLoaderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GaussRace - Gaussian Splat Racing".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GaussianSplattingPlugin)
        .add_plugins((
            SplatLoaderPlugin,
            GroundPlanePlugin,
            CarPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .run();
}

/// Sets up the initial scene with camera and lighting
fn setup_scene(mut commands: Commands) {
    // Spawn a 3D camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add ambient light
    commands.spawn((
        AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
            affects_lightmapped_meshes: true,
        },
    ));

    // Add directional light (sun)
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));
}
