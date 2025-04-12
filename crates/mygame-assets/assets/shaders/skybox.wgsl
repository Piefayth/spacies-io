#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0)
var sky_texture: texture_cube<f32>;
@group(2) @binding(1)
var sky_sampler: sampler;

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let color = textureSample(sky_texture, sky_sampler, in.world_normal).xyz;
    return vec4<f32>(color, 1.0);
}
