use bevy::{
    asset::RenderAssetUsages, color::palettes::css::{BLUE, RED}, pbr::{NotShadowCaster, NotShadowReceiver}, prelude::*, render::{
        mesh::MeshVertexBufferLayout, render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension}
    }
};
use mygame_assets::assets::GlobalAssets;
use mygame_render::camera::MainCamera;

use crate::game_state::GameState;

// A standalone plugin that spawns a checkered plane with a shader
pub (crate) struct ThrowawayPlugin;
impl Plugin for ThrowawayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<CheckerMaterial>::default())
           //.add_systems(Startup, spawn_shader_plane);
            .add_systems(OnEnter(GameState::Playing), add_sky_to_camera);
    }
}

fn add_sky_to_camera(
    mut commands: Commands,
    q_main_camera: Query<Entity, (With<MainCamera>, Without<EnvironmentMapLight>)>,
    global_assets: Res<GlobalAssets>,
) {
    for camera in &q_main_camera {
        commands.entity(camera).insert(EnvironmentMapLight {
            diffuse_map: global_assets.skybox_image.clone(),
            specular_map: global_assets.skybox_image.clone(),
            intensity: 3000.,
            ..default()
        })
        .with_child((
            NotShadowCaster,
            NotShadowReceiver,
            Name::new("Skybox"),
            Mesh3d(global_assets.skybox_mesh.clone()),
            MeshMaterial3d(global_assets.skybox_material.clone()),
        ));
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
