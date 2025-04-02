#import bevy_pbr::forward_io::{Vertex}
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

@group(2) @binding(0)
var<uniform> axis: u32;
@group(2) @binding(1)
var<uniform> start_color: vec4<f32>;
@group(2) @binding(2)
var<uniform> end_color: vec4<f32>;
@group(2) @binding(3)
var<uniform> start: f32;
@group(2) @binding(4)
var<uniform> end: f32;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) model_position: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
   
    // Pass the raw vertex position to the fragment shader
    out.model_position = vertex.position;
   
    out.position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var position_on_axis: f32;
    if (axis == 0u) {
        position_on_axis = in.model_position.x;
    } else if (axis == 1u) {
        position_on_axis = in.model_position.y;
    } else {
        position_on_axis = in.model_position.z;
    }
   
    // Normalize position between start and end values
    let range = end - start;
    let normalized_pos = clamp((position_on_axis - start) / range, 0.0, 1.0);
   
    // Create gradient by interpolating between colors
    let final_color = mix(start_color.rgb, end_color.rgb, normalized_pos);
   
    return vec4<f32>(final_color, 1.0);
}
