// Shader to render meshes with a texture sampler. Vertex color is obtained
// from the bound texture by sampling from the texture using vertex texture
// coordinates.

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
    @location(3) tex_coords: vec2<f32>,
}

// vertex shader

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = vertex.color;
    out.tex_coords = vertex.tex_coords;

    // Position modified by camera transformation, for display.
    out.view_position = camera.matrix * model_matrix.matrix * vec4<f32>(vertex.position, 1.0);

    // Rotate normal with body without translating.
    out.normal = normalize((model_matrix.matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    // World coordinates of vertex, after applying model transformation.
    let world_position = (model_matrix.matrix * vec4<f32>(vertex.position, 1.0));

    // Direction from point to light in world space.
    out.light_direction = normalize(light.position - world_position.xyz);

    return out;
}

// fragment shader

@group(3) @binding(0)
var diffuse_tex: texture_2d<f32>;

@group(3) @binding(1)
var diffuse_samp: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let use_light = (preferences.flags & 1u) > 0u;
    // TODO: Add correct handling for this in application.
    let use_texture = (preferences.flags & 2u) > 0u;

    var color: vec3<f32>;
    if use_texture {
        color = textureSample(diffuse_tex, diffuse_samp, in.tex_coords).xyz;
    } else {
        color = in.color;
    }

    if use_light {
        let ambient_strength = 0.05;
        let diffuse_strength = 0.95 * max(0.0, dot(in.light_direction, in.normal));
        let out_color = light.color * color;

        // Only ambient and diffuse lighting here for now.
        return vec4<f32>((ambient_strength + diffuse_strength) * out_color, 1.0);
    } else {

        return vec4<f32>(color, 1.0);
    }
}
