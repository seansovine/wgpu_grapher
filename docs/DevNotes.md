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

> `1.0 - 0.25*sqrt(sin(8.0*sqrt(x^2+z^2)) + 1.0)`

> `((sin(2*x))^2)^(3+2*(z*x+1.5))`

> `(cos(2*(x^2 + z^2)))^5/(x*z)`

The last one is cool in an Escher-like way.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/escher.png?raw=true"
		alt="drawing" width="800" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

### More examples

> `sin(2*x) + cos(3*z)`

> `2/(2 + cos(x*z))`

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

## Old Notes (pre-2026-07-25)

### More examples

_Example:_

- Function: `2.0*e^(5.0*(-(x-2.0)^2 - (z)^2))`
- Light position: `[3.0, 4.0, 0.0]`

_Example 2:_

- Function: `2.0*e^(5.0*(-(x)^2 - (z)^2))`
- Light position: `[0.0, 4.0, 0.0]`

_Example 3:_

- Function: `2.0*e^(-5.0*x^2)*e^(sin(2.0*z^2) - 1.0)`
- Light position: `[3.0, 4.0, 0.0]`

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/shadow_mapping_geometry_2026-01-10.png?raw=true" alt="drawing" width="700" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

_Example 4:_

- Function: `0.5*e^(-sin(4.0*(x^2 + z^2)))`
- Light position: `[3.0, 4.0, 0.0]`

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/radial_e_sin_square_2026-01-11.png?raw=true" alt="drawing" width="700" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

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
