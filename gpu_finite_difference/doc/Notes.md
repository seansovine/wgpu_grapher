# Wgpu Compute Finite Difference

This is a project to get started using Wgpu compute shaders. We will implement a
basic 2d finite difference solver with the ability to write the current state out
to an image file after running the solver for some timesteps.

Since our data is two-dimensional, we're using storage textures to hold the state
data. We could also use arbitrary buffers, but I think using textures simplifies
the setup a bit for our use case, and seems like a good place to start.

Wgpu compute has much fewer features than Nvidia's CUDA, and I've read that
achievable performance is generally lower than what you can get with CUDA. But,
Wgpu is a cross-platform API, and works with the integrated GPU on our lightweight
laptop. And Wgpu is a bit easier to work with than Vulkan, so we can try this out
here first, then maybe try similar things in Vulkan later.

## Next steps

_What we have now:_

What we have right now is the basic framework that creates the textures and the
compute pipeline, and runs a shader that just copies a multiple of one texture's
data into another texture, as a proof of concept and test that everything is setup
correctly.

_The next steps will be:_

1. ~~Add code to initialize the texture data.~~

2. ~~Add code to copy the texture data from the device and write it to an image file.~~

3. ~~Add a uniform binding to the pipeline to pass timestep and other params to the shader.~~

4. Add code in the shader to implement one timestep of the solver.

None of these steps should be especially complex to implement, since we've done similar
things in Wgpu render pipelines before.

Once we have the basic version we can look into some of the ways that compute shaders
can be optimized in Wgpu.

## Integration with Wgpu Grapher

_Plan:_

Once we have the basic solver working, our plan is to use it in Wgpu Grapher to do a
real-time wave equation simulation. We have a version of this in the old grapher crate,
but that version copies the vertex data to the device on every timestep, which is moving
a lot of data from the host to the GPU device. This version will be able to do the
computation directly on the device using the same vertex buffer that the render pipeline
is using. This will be very efficient. The only downside is that our graphics cards, like
most consumer cards, only support 32-bit floating point values.

_A few developer notes:_

For this integration we'll need to switch from using 2d textures to using a version of the
grapher vertex buffer that stores previous timestep data.

The device is free to run the compute and render pipelines in any order it wants. We
could add a call that blocks the host between the compute and render pass. But I think
I read somewhere that you can also reuse the same bindgroup for data that's used in both
pipelines, and Wgpu will insert a memory barrier that forces the pipelines to run one after
the other. This is similar to Vulkan, but Wgpu automates part of the process for you, it
seems.
