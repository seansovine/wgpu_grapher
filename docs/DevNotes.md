# Developer Notes

Next steps:

1. Add option to render lights and coordinate axes as scene objects.
2. Rework graph parameter GUI input and update handling.
3. Investigate ways to improve shadow mapping.
4. Add a proper file chooser.

Things to do later:

5. Port some mesh generation and lighting code from Vulkan Grapher.

## Known issues and TODOs

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

Shadow mapping for the floor mesh doesn't seem to be working correctly, and
there are some edge cases where shadow artifacts appear.

_Example:_

+ Function: `5.0*e^(5.0*(-(x-4.5)^2 - (z-3)^2))`
+ Light position: `[3.0, 4.0, 0.0]`

In this example we see what look like strange interactions between shadow mapping and
other lighting, in the region where the shadow would be just starting to appear.
I now think this is most likely due to aliasing and issues around the shape of
the mesh relative to the shape of the surface in certain areas of the graph.

_Example 2:_

+ Function: `2.0*e^(5.0*(-(x)^2 - (z)^2))`
+ Light position: `[0.0, 4.0, 0.0]`

This can be used to sanity check basic lighting and coordinate handling. As of now
everything seems to be working correctly except for shadow mapping.

_Example 3:_

This should be useful for debugging the geometry of shadow mapping.

+ Function: `2.0*e^(-5.0*x^2)*e^(sin(2.0*z^2) - 1.0)`
+ Light position: `[3.0, 4.0, 0.0]`

As the bumps move in the z-direction, we can see how the shadow varies.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/shadow_mapping_geometry_2026-01-10.png?raw=true" alt="drawing" width="700" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

_Example 4:_

I believe this example shows the effects of aliasing (and other factors) at some of
the shadow boundaries, especially where the shadow is created by our mesh's approximation
of a curved surface.

+ Function: `0.5*e^(-sin(4.0*(x^2 + z^2)))`
+ Light position: `[3.0, 4.0, 0.0]`

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/radial_e_sin_square_2026-01-11.png?raw=true" alt="drawing" width="700" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

_Plan:_

A checkbox for shadow mapping has been added, currently defaulting to off.

TODO: Look into ways to improve shadow mapping in the difficult cases.

## Lighting artifacts

_Example a:_

+ Functon: `max(0.0, sqrt(1.0 - x^2 + z^2))`
+ Light position: `[3.0, 4.0, 0.0]`

This example shows some lighting artifacts at the boundary where the shape gets
truncated to the `y = 0` plane. This is probably not surprising, as nearby triangles
can have _very_ different normals in this region. I will look into techniques for
handing things like this.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/truncated_cone_2026-01-11.png?raw=true" alt="drawing" width="700" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

## glTF handling improvements

### Handle transformations in node tree

We are now loading and composing the transformations along the way. We need some adjustments
to account for the fact that different models have their vertex coordinates scaled differently.
An idea is to compute the bounding box from the mesh as it is loaded and then add an additional
transformation that centers and scales the mesh so that it fills a standard sized bounding box
at the origin.

TODO: There seem to be a bug with the decomposed tranformations in some cases.

### Rework lighting for compatibility with glTF PBR material shading

We currently represent normals in world coordinates and use them directly
in lighting calculations. In the glTF model normals are represented in
the tangent space at each vertex, so each vertex also needs to have tangent and bitangent
vectors. We have implemented this approach  in the [Vulkan Grapher](https://github.com/seansovine/vulkan_grapher)
project. We could port some parts of the mesh generation and shader code from there to
this project.

### Efficiency

We should look at the efficiency of loading and rendering complex models. That's not something
we have looked at much in the program (yet!) as it has grown out of a program to render just a
graph into something more like a model viewer.
