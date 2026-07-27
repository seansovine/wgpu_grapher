# Developer Notes

## Example functions

These examples are interesting and/or highlight things that we want to understand or improve.

### Example

> `0.5*e^(-sin(x^2 + 4*z^2))`

This is a good example for shadow mapping quality. Increasing the coordinate coefficients
produces places where the curvature is high enough to start causing artifacts to appear.

### Example

> `2.0*e^(-5.0*x^2)*e^(sin(2.0*z^2) - 1.0)`

Interesting, and also a good shadow demonstration. The image shows the light position rendered as a small cube.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/shadow_debug.png?raw=true"
		alt="drawing" width="800" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

### Example

> `2.5*sin(15.0*sqrt((x + 0.01)*(x + 0.01) + (z + 0.01)*(z + 0.01))) / (15.0*sqrt((x + 0.01)*(x + 0.01) + (z + 0.01)*(z + 0.01))) + 0.5`

This is the radial sinc example used in the Vulkan Grapher repo.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/sinc_shadow_wireframe.png?raw=true"
		alt="drawing" width="800" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

### These are all interesting:

1. `1.0 - 0.25*sqrt(sin(8.0*sqrt(x^2+z^2)) + 1.0)`

2. `((sin(2*x))^2)^(3+2*(z*x+1.5))`

3. `(cos(2*(x^2 + z^2)))^5/(x*z)`

The last one is cool in an Escher-like way.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/escher.png?raw=true"
		alt="drawing" width="800" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

## Things of interest

### Curved contours and areas of high curvature

Similar issues are discussed in the Vulkan Grapher notes. The problem is that when the curvature
of the surface is high, the mesh badly approximates the surface, unless the mesh triangles are small
relative to the curvature. This also has an affect on lighting, because the normals tend to oscillate
in these areas due to the changing orientation of the triangles. However, this lighting issue isn't
as pronounced with the Phong lighting scheme used here as with the metallic-roughness PBR lighting
used in Vulkan Grapher.

### Surfaces with very fine details

There are a few things that go wrong when you have features of the object that change significantly
on small scales in device coordinate space. This is in part a fundamental limitation of any
computer renderer.

For example: If changes happen on scales finer than the mesh then they won't be rendered. But shrinking
the mesh size overloads the hardware with memory and processing demands. And, things that change on fine
spatial scales make higher demands on the precision of the numerical computations. These and other factors
result in the object and the mesh both being rendered inaccurately when there are fine details of the
object that change a significant amount.

There are likely good ways to handle these issues, or at least reasonable compromises and workarounds
that people have developed. We have tried some approaches to _smoothing out_ finer details after the
initial mesh is constructed.

### Possible shadow artifacts

There are some cases where there seem to be spurious shadows. It could be that they are correct but
unexpected based on the angles of the scene. We've added the ability to render the light as an object
in the scene, so that its position can be used to aid in debugging shadows and other lighting geometry
issues.

**Explanation:** The mesh coordinates go out of bounds of the shadow texture.

> **TODO:** Fix this by adjusting the projection matrix we use for shadow mapping.

## Old Notes (pre-2026-07-25)

This is a work in progress. There are a few known issues and some improvements
I'm planning to make in the near future.

Next steps:

1. Add option to render lights and coordinate axes as scene objects.
2. Rework graph parameter GUI input and update handling.
3. Investigate ways to improve shadow mapping.

Things to do later:

5. Port some mesh generation and lighting code from Vulkan Grapher.

### Coordinate axes and geometry debugging

To help debug lighting and other 3D rendering issues we will add some code to
optionally render scene objects for lights. It would also be nice to have some
coordinate axes that can be optionally displayed.

### Graph domain parameter updates

The GUI inputs for graph shift and scale are buggy. The way they're implemented
now also modifies the function object, so results in the graph being
regenerated on every change.

_Plan:_

We've currently disabled the function position and scale UI until we get the bugs
ironed out and decide how we want to handle updates to these. We may add a separate
window to update them, with an "apply" button.

### Shadow mapping

There are some edge cases where shadow artifacts appear.

_Example:_

- Function: `2.0*e^(5.0*(-(x-2.0)^2 - (z)^2))`
- Light position: `[3.0, 4.0, 0.0]`

It seems this is mostly caused by a combination of shadow aliasing
and the shape of our mesh not being optimal for certain parts of curved surfaces.
These effects are brought out more in a few cases.

_Example 2:_

- Function: `2.0*e^(5.0*(-(x)^2 - (z)^2))`
- Light position: `[0.0, 4.0, 0.0]`

This can be used to sanity check basic lighting and coordinate handling. As of now
everything seems to be working correctly in these areas.

_Example 3:_

This should be useful for debugging the geometry of shadow mapping.

- Function: `2.0*e^(-5.0*x^2)*e^(sin(2.0*z^2) - 1.0)`
- Light position: `[3.0, 4.0, 0.0]`

As the bumps move in the z-direction, we can see how the shadow varies.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/shadow_mapping_geometry_2026-01-10.png?raw=true" alt="drawing" width="700" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

_Example 4:_

I believe this example shows the effects of aliasing (and other factors) at some of
the shadow boundaries, especially where the shadow is created by our mesh's approximation
of a curved surface.

- Function: `0.5*e^(-sin(4.0*(x^2 + z^2)))`
- Light position: `[3.0, 4.0, 0.0]`

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/radial_e_sin_square_2026-01-11.png?raw=true" alt="drawing" width="700" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

_Plan:_

A checkbox for shadow mapping has been added, currently defaulting to off.

TODO: Look into ways to improve shadow mapping in the difficult cases.

## Lighting artifacts

_Example a:_

- Functon: `max(0.0, sqrt(1.0 - x^2 + z^2))`
- Light position: `[3.0, 4.0, 0.0]`

This example shows some lighting artifacts at the boundary where the shape gets
truncated to the `y = 0` plane. This is probably not surprising, as nearby triangles
can have _very_ different normals in this region. I will look into techniques for
handing things like this.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/truncated_cone_2026-01-11.png?raw=true" alt="drawing" width="700" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

## glTF handling improvements

### Rework lighting for compatibility with glTF PBR material shading

We currently represent normals in world coordinates and use them directly
in lighting calculations. In the glTF model normals are represented in
the tangent space at each vertex, so each vertex also needs to have tangent and bitangent
vectors. We have implemented this approach in the [Vulkan Grapher](https://github.com/seansovine/vulkan_grapher)
project. We could port some parts of the mesh generation and shader code from there to
this project.

### Efficiency

We should look more at the efficiency of loading and rendering complex models.
There are surely many more things that could be done here.
