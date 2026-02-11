@group(0) @binding(0) var eqn_data: texture_storage_2d<rgba32float, read_write>;

struct Uniform {
    timestep: u32,
};
@group(1) @binding(0) var<uniform> params_uniform: Uniform;


@compute @workgroup_size(8, 8)
fn advance(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // All three textures should have the same dimensions.
    let texture_dims: vec2<u32> = textureDimensions(eqn_data);
    if global_id.x > texture_dims.x || global_id.y > texture_dims.y {
        return;
    }
    // TODO: In our solver we'll need to respect the boundary conditions here.

    let coords = vec2<u32>(global_id.x, global_id.y);
    // This function always returns a vec4, regardless of texture dimensions.
    var value: vec4<f32> = textureLoad(eqn_data, coords);

    let t = params_uniform.timestep % 3;

    if (t == 0) {
        value[1] = 0.5 * value[0];
    } else if (t == 1) {
        value[2] = 1.5 * value[0];
    }
    textureStore(eqn_data, coords, value);

    // TODO: Implement one timestep of the equation solver.
}
