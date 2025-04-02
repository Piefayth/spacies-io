#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0)
var<uniform> tile_count: f32;
@group(2) @binding(1)
var<uniform> plane_size: f32;

// Add texture sampler with anisotropic filtering
@group(1) @binding(0)
var texture_sampler: sampler;

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    // Calculate the position on the plane
    let world_pos = in.world_position.xz;
    
    // Calculate derivatives for anisotropic filtering
    let dx = dpdx(world_pos);
    let dy = dpdy(world_pos);
    
    // Normalize position to 0-1 range across the whole plane
    let normalized_pos = (world_pos + plane_size / 2.0) / plane_size;
    
    // Calculate the frequency of the checker pattern
    let freq = tile_count / plane_size;
    
    // Compute the size of the filter kernel
    let filter_width = max(length(dx), length(dy)) * freq * 2.0;
    
    // Apply anti-aliasing for distant fragments
    var checker_result: f32 = 0.;
    
    if (filter_width < 1.0) {
        // When close, use the exact pattern
        let grid_pos = normalized_pos * tile_count;
        let tile_index = vec2<i32>(floor(grid_pos));
        let is_white = (tile_index.x + tile_index.y) % 2 == 0;
        checker_result = select(0.0, 1.0, is_white);
    } else {
        // For distant fragments, use analytical filtering
        let pi = 3.14159265359;
        let nx = sin(normalized_pos.x * tile_count * pi);
        let ny = sin(normalized_pos.y * tile_count * pi);
        let filter_factor = clamp(1.0 / filter_width, 0.0, 1.0);
        
        // Smooth transition between filtered and non-filtered
        let smoothed = 0.5 + 0.5 * sign(nx * ny);
        let filtered = 0.5; // Average of black and white
        checker_result = mix(filtered, smoothed, filter_factor);
    }
    
    // Choose color based on computed checker result
    let dark = vec3<f32>(0.1, 0.1, 0.1);
    let light = vec3<f32>(0.9, 0.9, 0.9);
    let color = mix(dark, light, checker_result);
    
    return vec4<f32>(color, 1.0);
}
