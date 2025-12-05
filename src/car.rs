//! Car/vehicle physics and controls
//!
//! This module provides a simple car that can drive around on the selected ground plane.

use bevy::prelude::*;

use crate::ground_plane::GroundPlane;

/// Plugin for car physics and controls
pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_car)
            .add_systems(Update, (
                handle_car_input,
                update_car_physics,
                update_camera_follow,
            ).chain());
    }
}

/// Component marking the player's car
#[derive(Component)]
pub struct Car {
    /// Current velocity (speed along forward direction)
    pub velocity: f32,
    /// Current steering angle in radians
    pub steering: f32,
    /// Maximum speed
    pub max_speed: f32,
    /// Acceleration rate
    pub acceleration: f32,
    /// Braking/deceleration rate
    pub brake_power: f32,
    /// Friction coefficient
    pub friction: f32,
    /// Maximum steering angle in radians
    pub max_steering: f32,
    /// Steering speed (how fast steering changes)
    pub steering_speed: f32,
    /// Car length (wheelbase) for turning calculations
    pub wheelbase: f32,
}

impl Default for Car {
    fn default() -> Self {
        Self {
            velocity: 0.0,
            steering: 0.0,
            max_speed: 30.0,
            acceleration: 15.0,
            brake_power: 25.0,
            friction: 5.0,
            max_steering: 0.6,
            steering_speed: 3.0,
            wheelbase: 2.0,
        }
    }
}

/// Component for the camera that follows the car
#[derive(Component)]
pub struct CarCamera {
    /// Offset from the car in local space
    pub offset: Vec3,
    /// How smoothly the camera follows (lower = smoother)
    pub smoothness: f32,
}

impl Default for CarCamera {
    fn default() -> Self {
        Self {
            offset: Vec3::new(0.0, 5.0, 12.0),
            smoothness: 5.0,
        }
    }
}

/// Spawn the player's car
fn spawn_car(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Car body (main box)
    let car_body = meshes.add(Cuboid::new(2.0, 0.8, 4.0));
    let car_top = meshes.add(Cuboid::new(1.6, 0.6, 2.0));
    let wheel = meshes.add(Cylinder::new(0.4, 0.3));
    
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        metallic: 0.8,
        perceptual_roughness: 0.3,
        ..default()
    });
    
    let top_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.15, 0.15),
        metallic: 0.6,
        perceptual_roughness: 0.4,
        ..default()
    });
    
    let wheel_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        metallic: 0.2,
        perceptual_roughness: 0.8,
        ..default()
    });

    // Spawn the car entity with children for body parts
    commands.spawn((
        Car::default(),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Visibility::default(),
    )).with_children(|parent| {
        // Car body
        parent.spawn((
            Mesh3d(car_body.clone()),
            MeshMaterial3d(body_material.clone()),
            Transform::from_xyz(0.0, 0.4, 0.0),
        ));
        
        // Car top (cabin)
        parent.spawn((
            Mesh3d(car_top),
            MeshMaterial3d(top_material),
            Transform::from_xyz(0.0, 1.0, 0.2),
        ));
        
        // Wheels
        let wheel_positions = [
            Vec3::new(-1.0, 0.0, 1.2),  // Front left
            Vec3::new(1.0, 0.0, 1.2),   // Front right
            Vec3::new(-1.0, 0.0, -1.2), // Rear left
            Vec3::new(1.0, 0.0, -1.2),  // Rear right
        ];
        
        for pos in wheel_positions {
            parent.spawn((
                Mesh3d(wheel.clone()),
                MeshMaterial3d(wheel_material.clone()),
                Transform::from_translation(pos)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ));
        }
    });

    // Mark the main camera as the car camera
    info!("Car spawned! Use WASD or arrow keys to drive.");
    info!("Press 'P' to enter plane selection mode.");
    info!("Press 'L' to load a Gaussian splat file.");
}

/// Handle keyboard input for car controls
fn handle_car_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut car_query: Query<&mut Car>,
    time: Res<Time>,
) {
    let Ok(mut car) = car_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    
    // Acceleration (W or Up)
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        car.velocity += car.acceleration * dt;
    }
    
    // Braking/Reverse (S or Down)
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        if car.velocity > 0.0 {
            car.velocity -= car.brake_power * dt;
        } else {
            car.velocity -= car.acceleration * 0.5 * dt; // Slower reverse
        }
    }
    
    // Steering (A/D or Left/Right)
    let steering_input = if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        1.0
    } else if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        -1.0
    } else {
        0.0
    };
    
    if steering_input != 0.0 {
        car.steering += steering_input * car.steering_speed * dt;
        car.steering = car.steering.clamp(-car.max_steering, car.max_steering);
    } else {
        // Return steering to center
        let return_speed = car.steering_speed * 2.0 * dt;
        if car.steering.abs() < return_speed {
            car.steering = 0.0;
        } else {
            car.steering -= car.steering.signum() * return_speed;
        }
    }
    
    // Apply friction
    if !keyboard.pressed(KeyCode::KeyW) && !keyboard.pressed(KeyCode::ArrowUp) &&
       !keyboard.pressed(KeyCode::KeyS) && !keyboard.pressed(KeyCode::ArrowDown) {
        let friction_decel = car.friction * dt;
        if car.velocity.abs() < friction_decel {
            car.velocity = 0.0;
        } else {
            car.velocity -= car.velocity.signum() * friction_decel;
        }
    }
    
    // Clamp velocity
    car.velocity = car.velocity.clamp(-car.max_speed * 0.3, car.max_speed);
}

/// Update car physics and position
fn update_car_physics(
    mut car_query: Query<(&mut Car, &mut Transform)>,
    ground_plane: Res<GroundPlane>,
    time: Res<Time>,
) {
    let Ok((mut car, mut transform)) = car_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    
    if car.velocity.abs() < 0.001 {
        return;
    }
    
    // Calculate the forward direction on the ground plane
    let forward = transform.forward();
    
    // Ackermann-like steering: turning radius depends on wheelbase and steering angle
    if car.steering.abs() > 0.001 {
        let turning_radius = car.wheelbase / car.steering.tan();
        let angular_velocity = car.velocity / turning_radius;
        
        // Rotate the car
        let rotation = Quat::from_axis_angle(ground_plane.up, angular_velocity * dt);
        transform.rotation = rotation * transform.rotation;
    }
    
    // Move the car forward
    let displacement = forward * car.velocity * dt;
    transform.translation += displacement;
    
    // Project the car onto the ground plane
    transform.translation = ground_plane.project_point(transform.translation);
    
    // Align the car's up vector with the ground plane normal
    let target_up = ground_plane.normal;
    let current_up = transform.up();
    
    if current_up.dot(target_up) < 0.999 {
        // Smoothly align to ground plane
        let align_rotation = Quat::from_rotation_arc(*current_up, target_up);
        let smoothed_rotation = Quat::IDENTITY.slerp(align_rotation, 10.0 * dt);
        transform.rotation = smoothed_rotation * transform.rotation;
    }
    
    // Keep car slightly above the ground
    transform.translation += ground_plane.normal * 0.5;
}

/// Update camera to follow the car
fn update_camera_follow(
    car_query: Query<&Transform, (With<Car>, Without<Camera3d>)>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    let Ok(car_transform) = car_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let smoothness = 5.0;
    
    // Calculate target camera position (behind and above the car)
    let offset = Vec3::new(0.0, 5.0, 12.0);
    let target_position = car_transform.translation 
        + car_transform.back() * offset.z 
        + car_transform.up() * offset.y;
    
    // Smoothly interpolate camera position
    camera_transform.translation = camera_transform.translation.lerp(
        target_position,
        smoothness * dt,
    );
    
    // Look at a point slightly ahead of the car
    let look_target = car_transform.translation + car_transform.forward() * 5.0;
    camera_transform.look_at(look_target, Vec3::Y);
}
