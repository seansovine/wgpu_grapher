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

struct PreferencesUniform {
    flags: u32,
}

@group(3) @binding(0)
var<uniform> preferences: PreferencesUniform;

// buffer structs

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) view_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) light_direction: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

// vertex shader

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = vertex.color;
    out.view_position = camera.matrix * model_matrix.matrix * vec4<f32>(vertex.position, 1.0);

    // rotate normal with body and pass through
    out.normal = (model_matrix.matrix * vec4<f32>(vertex.normal, 0.0)).xyz;
    // fragment shader gets direction from point to light in world space
    var world_position: vec3<f32> = (model_matrix.matrix * vec4<f32>(vertex.position, 1.0)).xyz;
    out.light_direction = normalize(light.position - world_position);

    return out;
}

// fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let use_light = (preferences.flags & 1) == 1;

    if use_light {
        let ambient_strength = 0.025;
        let ambient = ambient_strength * (light.color * in.color);

        let diffuse_strength = 0.4 * max(0.0, dot(in.light_direction, in.normal));
        let diffuse = diffuse_strength * (light.color * in.color);

        return vec4<f32>(ambient + diffuse, 1.0);
    } else {
        return vec4<f32>(in.color, 1.0);
    }

}
