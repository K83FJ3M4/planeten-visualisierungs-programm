@group(0) @binding(0) var<uniform> camera: Camera;

struct Camera {
    projection: mat4x4<f32>,
    position: vec4<f32>
}

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_position: vec3<f32>
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_position: vec3<f32>
}

struct FragmentOutput {
    @location(0) color: vec4<f32>
}

@vertex fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    let positions = array<vec3<f32>, 4>(
        vec3<f32>(-1.0, -1.0, 0.0),
        vec3<f32>(1.0, -1.0, 0.0),
        vec3<f32>(1.0, 1.0, 0.0),
        vec3<f32>(-1.0, 1.0, 0.0)
    );

    let indices = array<u32, 6>(
        0u, 2u, 1u,
        2u, 0u, 3u
    );

    let index = indices[input.vertex_index];
    var position = positions[index] * 100.0;
    let pos = vec4<f32>(position, 1.0);


    output.position = camera.projection * pos;
    output.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    output.world_position = position;
    output.uv = position.xy;
    
    return output;
}

@fragment fn fragment(input: FragmentInput) -> FragmentOutput {
    var output: FragmentOutput;

    let distance = length(camera.position.xyz - input.world_position);
    let grid_color = vec3<f32>(0.6, 0.6, 0.6);
    let opacity_x = select(0.0, 1.0, abs(input.uv.x % 1.0) < distance / 100.0);
    let opacity_y = select(0.0, 1.0, abs(input.uv.y % 1.0) < distance / 100.0);
    output.color = vec4<f32>(grid_color, opacity_x + opacity_y);
    
    return output;
}