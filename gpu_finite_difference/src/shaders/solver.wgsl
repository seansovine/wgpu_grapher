@group(0) @binding(0) var eqn_data: texture_storage_2d<rgba32float, read_write>;

struct Uniform {
    timestep: u32,
};
@group(1) @binding(0) var<uniform> params_uniform: Uniform;


@compute @workgroup_size(8, 8)
fn advance(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // All three textures should have the same dimensions.
    let texture_dims: vec2<u32> = textureDimensions(eqn_data);
    if global_id.x == 0 || global_id.x > texture_dims.x - 2
        || global_id.y == 0 || global_id.y > texture_dims.y - 2 {
        return;
    }
    // Don't update boundary points and stay within data range.

    let coords = vec2<u32>(global_id.x, global_id.y);

    var x_ij: vec4<f32> = textureLoad(eqn_data, coords);
    let x_imj: vec4<f32> = textureLoad(eqn_data, vec2<u32>(coords.x - 1, coords.y));
    let x_ipj: vec4<f32> = textureLoad(eqn_data, vec2<u32>(coords.x + 1, coords.y));
    let x_ijm: vec4<f32> = textureLoad(eqn_data, vec2<u32>(coords.x, coords.y - 1));
    let x_ijp: vec4<f32> = textureLoad(eqn_data, vec2<u32>(coords.x, coords.y + 1));

    let t = params_uniform.timestep % 3;
    let t_m1 = (params_uniform.timestep + 2) % 3;
    let t_m2 = (params_uniform.timestep + 1) % 3;

    const R: f32 = 0.35;

    let x_new: f32 = R * (x_imj[t_m1] + x_ipj[t_m1] + x_ijm[t_m1] + x_ijp[t_m1] - 4.0 * x_ij[t_m1])
                    + 2.0 * x_ij[t_m1] - x_ij[t_m2];

    x_ij[t] = x_new;
    textureStore(eqn_data, coords, x_ij);
}
