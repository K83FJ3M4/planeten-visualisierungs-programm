@group(0) @binding(0) var<uniform> camera: Camera;

struct Camera {
    projection: mat4x4<f32>,
    position: vec4<f32>
}

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) offset: vec4<f32>,
    @location(3) color_offset: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>
}

@vertex fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.projection * (input.position + input.offset);
    output.color = (input.color + input.color_offset) % 1.0;
    output.color.w = 1.0;
    return output;
}

@fragment fn fragment(input: FragmentInput) -> FragmentOutput {
    var output: FragmentOutput;
    output.color = input.color;
    return output;
}