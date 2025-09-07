// Shader to render meshes without using a texture sampler.
// Vertex color is obtained from its color coordinates.

struct MatrixUniform {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: MatrixUniform;

struct PreferencesUniform {
    flags: u32,
}

@group(0) @binding(1)
var<uniform> preferences: PreferencesUniform;

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
    @location(3) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) view_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) light_direction: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) reflected_light: vec3<f32>,
}

// vertex shader

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // position modified by camera position, for display
    out.view_position = camera.matrix * model_matrix.matrix * vec4<f32>(vertex.position, 1.0);
    out.color = vertex.color;

    // rotate normal with body and pass through
    out.normal = normalize((model_matrix.matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    // fragment shader gets direction from point to light in world space
    var world_position = (model_matrix.matrix * vec4<f32>(vertex.position, 1.0)).xyz;

    // from point to light in world coordinates
    out.light_direction = normalize(light.position - world_position);
    // light reflected across normal for specular lighting
    out.reflected_light = reflect(-out.light_direction, out.normal);

    return out;
}

// fragment shader

const SHININESS: f32 = 32.0;
const AMBIENT_CONTRIB: f32 = 0.025;
const DIFFUSE_CONTRIB: f32 = 0.4;
const SPECULAR_CONTRIB: f32 = 0.6;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let use_light = (preferences.flags & 1u) == 1u;

    if use_light {
        let diffuse_strength = DIFFUSE_CONTRIB * max(0.0, dot(in.light_direction, in.normal));
        let specular_strength = SPECULAR_CONTRIB * pow(max(0.0, dot(in.reflected_light, in.normal)), SHININESS);
        let out_color = light.color * in.color;

        return vec4<f32>((AMBIENT_CONTRIB + diffuse_strength + specular_strength) * out_color, 1.0);
    } else {
        return vec4<f32>(in.color, 0.8);
    }
}
