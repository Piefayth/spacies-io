use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        mesh::MeshVertexBufferLayout,
    },
};

// A standalone plugin that spawns a checkered plane with a shader
pub (crate) struct ThrowawayPlugin;
impl Plugin for ThrowawayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<CheckerMaterial>::default())
           .add_systems(Startup, spawn_shader_plane);
    }
}

// Custom material for the checker pattern
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct CheckerMaterial {
    #[uniform(0)]
    tile_count: f32,
    #[uniform(1)]
    plane_size: f32,
}

impl Material for CheckerMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/checker.wgsl".into()
    }
}

// Spawn a single plane with the checker shader
fn spawn_shader_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CheckerMaterial>>,
) {
    // Configuration
    let plane_size = 1000.0; // Size of the entire plane
    let tile_count = 20.0; // Number of tiles along each axis

    // Create a single plane mesh
    let plane_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(plane_size / 2.0)));
    
    // Create the checker material
    let checker_material = materials.add(CheckerMaterial {
        tile_count,
        plane_size,
    });
    
    // Spawn the plane with the shader
    commands.spawn((
        Mesh3d(plane_mesh),
        MeshMaterial3d(checker_material),
        Transform::from_translation(Vec3::ZERO)
    ));
    
    println!("Spawned a checkered plane with shader (equivalent to {} tiles)", 
             tile_count as i32 * tile_count as i32);
}
