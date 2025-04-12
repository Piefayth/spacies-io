use bevy::{
    asset::RenderAssetUsages, color::palettes::css::{BLUE, RED}, prelude::*, render::{
        mesh::MeshVertexBufferLayout, render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension}
    }
};

pub fn hemispherical_gradient(
    top_color: Color,
    bottom_color: Color,
) -> Image {
    let top_color: Srgba = top_color.into();
    let bottom_color: Srgba = bottom_color.into();
    let mid_color = (top_color + bottom_color) / 2.0;
    
    Image {
        texture_view_descriptor: Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        }),
        ..Image::new(
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 6,
            },
            TextureDimension::D2,
            [
                mid_color,
                mid_color,
                top_color,
                bottom_color,
                mid_color,
                mid_color,
            ]
            .into_iter()
            .flat_map(Srgba::to_u8_array)
            .collect(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}
