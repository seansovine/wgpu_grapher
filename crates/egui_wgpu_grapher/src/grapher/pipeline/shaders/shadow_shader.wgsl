// Shader for efficiently computing shadow map.

struct MatrixUniform {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> light_view: MatrixUniform;

@group(1) @binding(0)
var<uniform> model_matrix: MatrixUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
) -> @builtin(position) vec4<f32> {
    return light_view.matrix * model_matrix.matrix * vec4<f32>(vertex.position, 1.0);
}
