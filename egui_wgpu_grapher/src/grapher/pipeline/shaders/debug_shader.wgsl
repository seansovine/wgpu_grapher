// Render hard-coded cube in scene light position.

struct MatrixUniform {
    matrix: mat4x4<f32>,
}

struct LightUniform {
    position: vec3<f32>,
    color: vec3<f32>,
}

const LIGHT_CUBE_POS = array<vec3<f32>, 8>(
    vec3<f32>(-0.5, -0.5, -0.5), // 0
    vec3<f32>(0.5, -0.5, -0.5),  // 1
    vec3<f32>(0.5, 0.5, -0.5),   // 2
    vec3<f32>(-0.5, 0.5, -0.5),  // 3
    vec3<f32>(-0.5, -0.5, 0.5),  // 4
    vec3<f32>(0.5, -0.5, 0.5),   // 5
    vec3<f32>(0.5, 0.5, 0.5),    // 6
    vec3<f32>(-0.5, 0.5, 0.5),   // 7
);

const LIGHT_CUBE_INDICES = array<u32, 36>(
    4, 5, 6, 4, 6, 7, // Front face
    1, 0, 3, 1, 3, 2, // Back face
    0, 4, 7, 0, 7, 3, // Left face
    5, 1, 2, 5, 2, 6, // Right face
    7, 6, 2, 7, 2, 3, // Top face
    0, 1, 5, 0, 5, 4, // Bottom face
);

@group(0) @binding(0)
var<uniform> camera: MatrixUniform;

@group(1) @binding(0)
var<uniform> light: LightUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

const LIGHT_SCALE: f32 = 0.1;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.matrix * vec4<f32>(LIGHT_SCALE * LIGHT_CUBE_POS[LIGHT_CUBE_INDICES[in_vertex_index]] + light.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(light.color, 1.0);
}
