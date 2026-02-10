@group(0) @binding(0) var eqn_timestep_0_data: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var eqn_timestep_1_data: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var eqn_timestep_2_data: texture_storage_2d<r32float, read_write>;

struct Uniform {
    timestep: u32,
};
@group(1) @binding(0) var<uniform> params_uniform: Uniform;


@compute @workgroup_size(8, 8)
fn advance(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // All three textures should have the same dimensions.
    let texture_dims: vec2<u32> = textureDimensions(eqn_timestep_0_data);
    if global_id.x > texture_dims.x || global_id.y > texture_dims.y {
        return;
    }
    // TODO: In our solver we'll need to respect the boundary conditions here.

    let coords = vec2<u32>(global_id.x, global_id.y);
    // This function always returns a vec4, regardless of texture dimensions.
    var value: vec4<f32> = textureLoad(eqn_timestep_0_data, coords);

    if (params_uniform.timestep % 2 ==  0) {
        // Update value as a test.
        value.x *= 0.25;
        textureStore(eqn_timestep_1_data, coords, value);
    } else {
        // Update value as a test.
        value.x *= 1.25;
        textureStore(eqn_timestep_2_data, coords, value);
    }

    // TODO:
    //  - Add uniform binding for current timestep & other params.
    //  - Implement one timestep of the equation solver.
}
