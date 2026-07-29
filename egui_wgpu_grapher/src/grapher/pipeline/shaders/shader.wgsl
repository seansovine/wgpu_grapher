// Shader to render meshes with vertex color given by vertex color properties.

struct MatrixUniform {
    matrix: mat4x4<f32>,
}

struct PreferencesUniform {
    flags: u32,
}

struct LightUniform {
    position: vec3<f32>,
    color: vec3<f32>,
}

// Vertex shader.

@group(0) @binding(0)
var<uniform> camera: MatrixUniform;
@group(0) @binding(1)
var<uniform> preferences: PreferencesUniform;

@group(1) @binding(0)
var<uniform> model_matrix: MatrixUniform;

@group(2) @binding(0)
var<uniform> light: LightUniform;

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
    @location(5) is_selected: u32,
}

const SHOW_SELECTION:  bool = false;
const SELECTED_POINT        = vec2<f32>(0.0, 0.0);
const SELECTION_RADIUS: f32 = 0.01;

@vertex
fn vs_main(vertex: VertexInput, @builtin(vertex_index) vert_ind: u32) -> VertexOutput {
    var out: VertexOutput;
    out.color = vertex.color;

    // Position modified by camera transformation, for display.
    out.view_position = camera.matrix * model_matrix.matrix * vec4<f32>(vertex.position, 1.0);
    // Rotate normal with body without translating.
    out.normal = normalize((model_matrix.matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    // World coordinates of vertex, after applying model transformation.
    out.world_position = (model_matrix.matrix * vec4<f32>(vertex.position, 1.0));

    // Direction from point to light in world space.
    out.light_direction = normalize(light.position - out.world_position.xyz);
    // Light reflected across normal for specular lighting.
    out.reflected_light = reflect(-out.light_direction, out.normal);

    if SHOW_SELECTION {
        out.is_selected = u32(distance(SELECTED_POINT, out.view_position.xy / out.view_position.w) <= SELECTION_RADIUS);
    } else {
        out.is_selected = 0;
    }

    return out;
}

// Fragment shader.

@group(3) @binding(0)
var shadow_texture: texture_depth_2d;
@group(3) @binding(1)
var shadow_sampler: sampler_comparison;
@group(3) @binding(2)
var<uniform> light_view: MatrixUniform;

struct LightSettings {
    shininess: f32,
    ambient_v: f32,
    diffuse_v: f32,
    speculr_v: f32,
}

const LIGHT_SETTINGS = LightSettings(
    32.0,  // shininess
    0.05,  // ambient
    0.4,   // diffuse
    0.6,   // specular
);

// Modified from the Wgpu shadow example.
fn get_shadow(world_position: vec4<f32>) -> f32 {
    // To convert device coords to texture coords; reverse is done
    // automatically when rendering to depth buffer.
    const flip_correction = vec2<f32>(0.5, -0.5);

    // To normalize homogenous coords so that w = 1.0; light view
    // projection may leave them un-normalized.
    let proj_correction = 1.0 / world_position.w;

    // Coordinates in depth buffer corresponding to this point.
    let shadow_tex_coords = world_position.xy *
        proj_correction * flip_correction + vec2<f32>(0.5, 0.5);

    return textureSampleCompare(shadow_texture, shadow_sampler,
        shadow_tex_coords, world_position.z * proj_correction);
}

// We aren't currently depth sorting, but convenient for debugging.
const DEFAULT_ALPHA: f32 = 1.0;

// To test primitive highlighting.
const SELECTED_TRI: u32 = 0x7fffffffu;
const SELECTED_COLOR    = vec4<f32>(0.0, 1.0, 0.0, DEFAULT_ALPHA);

// Preference bits.

const LIGHT_BIT:  u32 = 1u;
const SHADOW_BIT: u32 = 4u;

@fragment
fn fs_main(@builtin(primitive_index) prim_index: u32, in: VertexOutput) -> @location(0) vec4<f32> {
    let selected_t: f32 = select(0.0, 1.0, bool(in.is_selected) || abs(prim_index - SELECTED_TRI) < 100);

    if (preferences.flags & LIGHT_BIT) > 0 {
        let shadow = select(get_shadow(light_view.matrix * in.world_position), 1.0, (preferences.flags & SHADOW_BIT) == 0);
        let diffuse_strength = shadow *
            LIGHT_SETTINGS.diffuse_v * max(0.0, dot(in.light_direction, in.normal));
        let specular_strength = shadow *
            LIGHT_SETTINGS.speculr_v * pow(max(0.0, dot(in.reflected_light, in.normal)), LIGHT_SETTINGS.shininess);

        let out_color = vec4<f32>((LIGHT_SETTINGS.ambient_v + diffuse_strength + specular_strength) * (light.color * in.color), DEFAULT_ALPHA);

        // Apply Phong illumination model.
        return selected_t * SELECTED_COLOR + (1.0 - selected_t) * out_color;

    } else {
        // We're use alpha transparency when lighting is disabled. This is experimental
        // because we aren't depth sorting our primitives, so it won't be correct.
        return selected_t * SELECTED_COLOR + (1.0 - selected_t) * vec4<f32>(in.color, 0.8);
    }
}
