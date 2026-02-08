@group(0) @binding(0) var eqn_timestep_0_data: texture_2d<f32>;
@group(0) @binding(1) var eqn_timestep_1_data: texture_2d<f32>;
@group(0) @binding(2) var eqn_timestep_2_data: texture_2d<f32>;

@compute @workgroup_size(8, 8)
fn advance(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // TODO: To start just sample and update texture. Then
    //       we'll implement one timestep of the equation solver.
}
