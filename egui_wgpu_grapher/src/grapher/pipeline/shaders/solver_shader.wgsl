const WIDTH: f32 = 0.9;

const QUAD_VERTS: array<vec4f, 4> = array(
    vec4f(-WIDTH, -WIDTH, 0.5, 1.0),
    vec4f( WIDTH, -WIDTH, 0.5, 1.0),
    vec4f( WIDTH,  WIDTH, 0.5, 1.0),
	vec4f(-WIDTH,  WIDTH, 0.5, 1.0),
);

const QUAD_TEX_COORDS: array<vec2f, 4> = array(
    vec2f(0.0, 1.0),
    vec2f(1.0, 1.0),
    vec2f(1.0, 0.0),
    vec2f(0.0, 0.0)
);

struct Uniform {
    timestep: u32,
    aspect_ratio: f32,
};

@group(0) @binding(0) var<uniform> params_uniform: Uniform;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) tex_coords: vec2f,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_index: u32
) -> VertexOutput {
    var out: VertexOutput;
    out.position = QUAD_VERTS[in_index];
    out.position.x *= params_uniform.aspect_ratio;
    out.tex_coords = QUAD_TEX_COORDS[in_index];
    return out;
}

@group(1) @binding(0) var data_texture: texture_2d<f32>;
@group(1) @binding(1) var data_sampler: sampler;

const TEXTURE_MAX_VAL: f32 = 255.0;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let t = params_uniform.timestep % 3;
    let sample = textureSample(data_texture, data_sampler, in.tex_coords)[t] / TEXTURE_MAX_VAL;
	return vec4f(sample, sample, sample, 1.0);
}
