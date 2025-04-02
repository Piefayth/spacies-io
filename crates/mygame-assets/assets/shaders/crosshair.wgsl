#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0)
var<uniform> color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize UVs to [-1, 1] range
    let uv = in.uv * 2.0 - 1.0;
    
    // Calculate distance from edges for a crisp 1px outline
    let edge_thickness = 0.05;
    
    // Create sharp edge mask for x and y edges
    let x_dist = abs(abs(uv.x) - 1.0);
    let y_dist = abs(abs(uv.y) - 1.0);
    
    // Use step for crisp edges
    let x_edge = step(x_dist, edge_thickness);
    let y_edge = step(y_dist, edge_thickness);
    
    // Combine edges using max
    let edge_mask = max(x_edge, y_edge);
    
    // Multiply color by edge mask
    return color * edge_mask;
}
