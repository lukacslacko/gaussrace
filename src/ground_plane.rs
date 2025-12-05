//! Ground plane selection and management
//!
//! This module allows users to define a ground plane within the Gaussian splat
//! that the car will drive on.

use bevy::prelude::*;

/// Plugin for ground plane selection and management
pub struct GroundPlanePlugin;

impl Plugin for GroundPlanePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GroundPlane>()
            .add_systems(Update, handle_plane_selection)
            .add_systems(Update, visualize_ground_plane);
    }
}

/// Resource defining the ground plane parameters
/// The plane is defined by a point and a normal vector
#[derive(Resource)]
pub struct GroundPlane {
    /// A point on the plane
    pub origin: Vec3,
    /// The normal vector of the plane (unit vector)
    pub normal: Vec3,
    /// The up direction for the plane (perpendicular to forward on the plane)
    pub up: Vec3,
    /// Whether the plane has been selected
    pub is_selected: bool,
}

impl Default for GroundPlane {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            normal: Vec3::Y, // Default to horizontal plane
            up: Vec3::Y,
            is_selected: false,
        }
    }
}

impl GroundPlane {
    /// Create a new ground plane from three points
    pub fn from_three_points(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(v2).normalize();
        
        // Ensure the normal points "up" (positive Y component)
        let normal = if normal.y < 0.0 { -normal } else { normal };
        
        Self {
            origin: p1,
            normal,
            up: normal,
            is_selected: true,
        }
    }

    /// Project a world position onto the ground plane
    pub fn project_point(&self, point: Vec3) -> Vec3 {
        let to_point = point - self.origin;
        let distance = to_point.dot(self.normal);
        point - distance * self.normal
    }

    /// Get the height (distance from plane) at a given point
    pub fn height_at(&self, point: Vec3) -> f32 {
        let to_point = point - self.origin;
        to_point.dot(self.normal)
    }
}

/// Component for plane selection mode markers
#[derive(Component)]
struct PlaneSelectionMarker(usize);

/// Resource to track plane selection state
#[derive(Resource, Default)]
struct PlaneSelectionState {
    points: Vec<Vec3>,
    active: bool,
}

/// Handle plane selection input
fn handle_plane_selection(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut ground_plane: ResMut<GroundPlane>,
    mut selection_state: Local<PlaneSelectionState>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    markers: Query<Entity, With<PlaneSelectionMarker>>,
) {
    // Toggle plane selection mode with 'P' key
    if keyboard.just_pressed(KeyCode::KeyP) {
        selection_state.active = !selection_state.active;
        if selection_state.active {
            info!("Plane selection mode ACTIVE - Click 3 points to define the ground plane");
            selection_state.points.clear();
            // Remove old markers
            for entity in markers.iter() {
                commands.entity(entity).despawn();
            }
        } else {
            info!("Plane selection mode INACTIVE");
        }
    }

    // Reset plane with 'R' key
    if keyboard.just_pressed(KeyCode::KeyR) {
        *ground_plane = GroundPlane::default();
        selection_state.points.clear();
        for entity in markers.iter() {
            commands.entity(entity).despawn();
        }
        info!("Ground plane reset to default");
    }

    if !selection_state.active {
        return;
    }

    // Handle mouse clicks for point selection
    if mouse_button.just_pressed(MouseButton::Left) {
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };
        let Ok(window) = windows.single() else {
            return;
        };

        if let Some(cursor_pos) = window.cursor_position() {
            // Cast a ray from the camera through the cursor position
            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
                // For now, intersect with a horizontal plane at y=0 or the current plane
                let plane_normal = ground_plane.normal;
                let plane_origin = ground_plane.origin;
                
                let denom = plane_normal.dot(*ray.direction);
                if denom.abs() > 0.0001 {
                    let t = (plane_origin - ray.origin).dot(plane_normal) / denom;
                    if t > 0.0 {
                        let hit_point = ray.origin + *ray.direction * t;
                        
                        selection_state.points.push(hit_point);
                        info!("Selected point {}: {:?}", selection_state.points.len(), hit_point);
                        
                        // Spawn a visual marker
                        commands.spawn((
                            Mesh3d(meshes.add(Sphere::new(0.2))),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: Color::srgb(1.0, 0.0, 0.0),
                                emissive: LinearRgba::rgb(1.0, 0.0, 0.0),
                                ..default()
                            })),
                            Transform::from_translation(hit_point),
                            PlaneSelectionMarker(selection_state.points.len()),
                        ));

                        // If we have 3 points, create the plane
                        if selection_state.points.len() >= 3 {
                            *ground_plane = GroundPlane::from_three_points(
                                selection_state.points[0],
                                selection_state.points[1],
                                selection_state.points[2],
                            );
                            selection_state.active = false;
                            info!("Ground plane defined! Normal: {:?}", ground_plane.normal);
                        }
                    }
                }
            }
        }
    }
}

/// Visualize the ground plane when selected
fn visualize_ground_plane(
    mut gizmos: Gizmos,
    ground_plane: Res<GroundPlane>,
) {
    if !ground_plane.is_selected {
        return;
    }

    // Draw a grid on the ground plane
    let origin = ground_plane.origin;
    let normal = ground_plane.normal;
    
    // Calculate tangent vectors
    let tangent1 = if normal.y.abs() < 0.9 {
        normal.cross(Vec3::Y).normalize()
    } else {
        normal.cross(Vec3::X).normalize()
    };
    let tangent2 = normal.cross(tangent1).normalize();

    let grid_size = 20.0;
    let grid_spacing = 2.0;
    let color = Color::srgba(0.0, 1.0, 0.0, 0.3);

    let steps = (grid_size / grid_spacing) as i32;
    for i in -steps..=steps {
        let offset = i as f32 * grid_spacing;
        
        // Lines along tangent1
        let start = origin + tangent1 * offset - tangent2 * grid_size;
        let end = origin + tangent1 * offset + tangent2 * grid_size;
        gizmos.line(start, end, color);
        
        // Lines along tangent2
        let start = origin + tangent2 * offset - tangent1 * grid_size;
        let end = origin + tangent2 * offset + tangent1 * grid_size;
        gizmos.line(start, end, color);
    }

    // Draw the normal vector
    gizmos.arrow(origin, origin + normal * 3.0, Color::srgb(0.0, 0.0, 1.0));
}
