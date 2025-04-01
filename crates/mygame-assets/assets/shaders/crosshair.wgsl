#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0)
var<uniform> color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - 1.0;
    let center_dist = length(uv);
    
    let glow = 1.0 - smoothstep(0.4, 0.5, center_dist);
    
    let crosshair_x = smoothstep(0.05, 0.0, abs(uv.x));
    let crosshair_y = smoothstep(0.05, 0.0, abs(uv.y));
    let crosshair = max(crosshair_x, crosshair_y);
    
    let alpha = max(glow * 0.5, crosshair);
    
    return vec4<f32>(color.rgb, alpha * color.a);
}
