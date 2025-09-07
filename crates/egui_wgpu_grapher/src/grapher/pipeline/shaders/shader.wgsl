// Shader to render meshes without using a texture sampler.
// Vertex color is obtained from its color coordinates.

// Uniforms.

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

// Input/output buffer structures.

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
    @location(4) world_position: vec4<f32>,
}

// Vertex shader.

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // position modified by camera position, for display
    out.view_position = camera.matrix * model_matrix.matrix * vec4<f32>(vertex.position, 1.0);
    out.color = vertex.color;

    // rotate normal with body without translating
    out.normal = normalize((model_matrix.matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    // fragment shader gets direction from point to light in world space
    out.world_position = (model_matrix.matrix * vec4<f32>(vertex.position, 1.0));

    // from point to light in world coordinates
    out.light_direction = normalize(light.position - out.world_position.xyz);
    // light reflected across normal for specular lighting
    out.reflected_light = reflect(-out.light_direction, out.normal);

    return out;
}

// Fragment shader.

@group(3) @binding(0)
var shadow_texture: texture_depth_2d;
@group(3) @binding(1)
var shadow_sampler: sampler_comparison;

@group(4) @binding(0)
var<uniform> light_view: MatrixUniform;

struct LightSettings {
    shininess: f32,
    ambient_v: f32,
    diffuse_v: f32,
    speculr_v: f32,
}

const LIGHT_SETTINGS = LightSettings(
    32.0,  // shininess
    0.05, // ambient
    0.4,   // diffuse
    0.6,   // specular
);

// Modified from the WGPU shadow example.
fn get_shadow(world_position: vec4<f32>) -> f32 {
    // From the WGPU comments:
    //  "compensate for the Y-flip difference between the NDC and texture coordinates"
    let flip_correction = vec2<f32>(0.5, -0.5);
    // The light view matrix alters the w coordinate.
    let proj_correction = 1.0 / world_position.w;
    let shadow_tex_coords = world_position.xy * proj_correction * flip_correction + vec2<f32>(0.5, 0.5);
    return textureSampleCompareLevel(shadow_texture, shadow_sampler, shadow_tex_coords, world_position.z * proj_correction);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let use_light = (preferences.flags & 1u) == 1u;

    if use_light {
        // Apply Phong illumination w/ directions from vertex shader.
        let shadow = get_shadow(light_view.matrix * in.world_position);
        let diffuse_strength = shadow * LIGHT_SETTINGS.diffuse_v * max(0.0, dot(in.light_direction, in.normal));
        let specular_strength = shadow * LIGHT_SETTINGS.speculr_v * pow(max(0.0, dot(in.reflected_light, in.normal)), LIGHT_SETTINGS.shininess);
        let out_color = light.color * in.color;
        return vec4<f32>((LIGHT_SETTINGS.ambient_v + diffuse_strength + specular_strength) * out_color, 1.0);
    } else {
        return vec4<f32>(in.color, 0.8);
    }
}
