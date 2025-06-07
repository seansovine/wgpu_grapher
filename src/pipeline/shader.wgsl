// uniforms

struct MatrixUniform {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: MatrixUniform;

@group(1) @binding(0)
var<uniform> model_matrix: MatrixUniform;

struct LightUniform {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> light: LightUniform;

// buffer structs

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) frag_position: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

// vertex shader

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = model.color;
    out.clip_position = camera.matrix * model_matrix.matrix * vec4<f32>(model.position, 1.0);

    // pass normal through
    out.normal = model.normal;
    // fragment shader gets position in world space
    out.frag_position = (model_matrix.matrix * vec4<f32>(model.position, 1.0)).xyz;

    return out;
}

// fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient_strength = 1.0;
    let ambient_color = light.color * ambient_strength;
    let result = ambient_color * in.color;

    return vec4<f32>(result, 1.0);
}
