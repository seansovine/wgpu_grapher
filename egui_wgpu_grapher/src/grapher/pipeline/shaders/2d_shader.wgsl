const WIDTH: f32 = 0.6;

const QUAD_VERTS: array<vec4f, 4> = array(
    vec4f(-WIDTH, -WIDTH, 0.5, 1.0),
    vec4f( WIDTH, -WIDTH, 0.5, 1.0),
    vec4f( WIDTH,  WIDTH, 0.5, 1.0),
	vec4f(-WIDTH,  WIDTH, 0.5, 1.0),
);

@vertex
fn vs_main(
    @builtin(vertex_index) in_index: u32
) -> @builtin(position) vec4<f32> {
    return QUAD_VERTS[in_index];
}

// TODO: Bind RGBA f32 texture and use uniform to sample appropriate channel.

@fragment
fn fs_main(@builtin(position) in_position: vec4<f32>) -> @location(0) vec4<f32> {
	return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
