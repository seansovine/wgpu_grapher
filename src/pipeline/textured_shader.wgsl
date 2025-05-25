// vertex shader

struct MatrixUniform {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: MatrixUniform;

@group(1) @binding(0)
var<uniform> model_matrix: MatrixUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.matrix * model_matrix.matrix * vec4<f32>(model.position, 1.0);
    return out;
}

// fragment shader

@group(2) @binding(0)
var diffuse_tex: texture_2d<f32>;

@group(2) @binding(1)
var diffuse_samp: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(diffuse_tex, diffuse_samp, in.tex_coords);
}
