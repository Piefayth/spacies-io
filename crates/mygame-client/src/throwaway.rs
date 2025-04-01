use bevy::{prelude::*};

// Main function to demonstrate usage
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ThrowawayPlugin)
        .run();
}

// A standalone plugin that spawns a large checkered plane
pub struct ThrowawayPlugin;

impl Plugin for ThrowawayPlugin {
    fn build(&self, app: &mut App) {
        app
           .add_systems(Startup, spawn_checkered_plane);
    }
}

// Spawn the checkered plane using custom materials instead of PBR
fn spawn_checkered_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Configuration
    let plane_size = 1000.0; // Size of the entire plane
    let tile_count = 20; // Number of tiles along each axis
    let tile_size = plane_size / tile_count as f32;
   
    // Create materials for the checker pattern
    let white_material = materials.add(StandardMaterial::from_color(Color::linear_rgb(0.9, 0.9, 0.9)));
    let black_material = materials.add(StandardMaterial::from_color(Color::linear_rgb(0.1, 0.1, 0.1)));
   
    // Create a single tile mesh that we'll reuse
    let tile_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(tile_size / 2.)));
   
    // Calculate the starting position (top-left corner of the plane)
    let start_x = -plane_size / 2.0 + tile_size / 2.0;
    let start_z = -plane_size / 2.0 + tile_size / 2.0;
   
    // Spawn all the tiles
    for x in 0..tile_count {
        for z in 0..tile_count {
            let position = Vec3::new(
                start_x + x as f32 * tile_size,
                0.0,
                start_z + z as f32 * tile_size,
            );
           
            // Choose material based on checkerboard pattern
            let material = if (x + z) % 2 == 0 {
                white_material.clone()
            } else {
                black_material.clone()
            };
           
            commands.spawn((
                Mesh3d(tile_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(position)
            ));
        }
    }
   
    println!("Spawned a checkered plane with {} tiles", tile_count * tile_count);
}
