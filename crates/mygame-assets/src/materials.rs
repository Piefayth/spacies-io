use bevy::{prelude::*, render::render_resource::{AsBindGroup, ShaderRef, ShaderType}};

pub (crate) struct SharedMaterialPlugin;

impl Plugin for SharedMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<GradientMaterial>::default())
            .register_type::<GradientMaterial>()
            .register_asset_reflect::<GradientMaterial>();

        app.add_plugins(MaterialPlugin::<SkyboxMaterial>::default());
    }
}


#[derive(Asset, AsBindGroup, Debug, Clone, Default, Reflect)]
pub struct GradientMaterial {
    #[uniform(0)]
    pub axis: u32,  // 0 = X, 1 = Y, 2 = Z
    
    #[uniform(1)]
    pub start_color: LinearRgba,
    
    #[uniform(2)]
    pub end_color: LinearRgba,

    #[uniform(3)]
    pub start: f32,
    
    #[uniform(4)]
    pub end: f32,
}

impl Material for GradientMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/gradient.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/gradient.wgsl".into()
    }
}

#[derive(AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct SkyboxMaterial {
    #[texture(0, dimension = "cube")]
    #[sampler(1)]
    pub sky_texture: Handle<Image>,
}

impl Material for SkyboxMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/skybox.wgsl".into()
    }
}
