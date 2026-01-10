# Developer Notes

Next steps:

1. Add option to render light in scene as a sphere for debugging.
2. Rework GUI state and parameter handling.
3. Rework shadow mapping.

Things to do later:

4. Enable multisampling.
5. Port mesh format and lighting from Vulkan Grapher.

## Known issues and TODOs

### Parameter input bugs

The behavior of the graph relative to GUI parameter inputs is very buggy.
Likely we need to rework the way these values are stored and used into
a more sane structure.

_Plan:_

We've currently disabled the function position and scale UI until we get the bugs
ironed out and decide how we want to handle updates to these going forward.

### Shadow mapping bugs

Shadow mapping for the floor mesh doesn't seem to be working correctly, and
there are some edge cases where strange artifacts appear.

To help debug this and other 3D rendering issues we will add some code to
optionally render scene objects for light position, coordinate axes, etc.

_Example:_

+ Light position: `[3.0, 4.0, 0.0]`
+ Function: `5.0*e^(5.0*(-(x-4.5)^2 - (z-3)^2))`

In this example we see some strange interactions between shadow mapping and
other lighting, in the region where the shadow would be just starting to appear.
This could be due to issues like overflow or negative values appearing in the
shader, but I'm not sure.

_Example 2:_

+ Function: `2.0*e^(5.0*(-(x)^2 - (z)^2))`

This can be used to sanity check basic lighting and coordinate handling. As of now
everything seems to be working correctly except for shadow mapping.

_Plan:_

A checkbox for shadow mapping has been added, defaulting to off until we get shadow
mapping fixed.

TODO: Rework shadow mapping and then verify its correctness.

## glTF handling improvements

### Handle transformations in node tree

Most glTF scenes / models of any complexity have a hierachical structure of 3D
transformations attached to their hierarchy of nodes and their meshes. We need
to store these transformations as the node hierarchy is loaded so they can be
composed properly to render the meshes in the correct positions, scales, and
orientations. We should have much of what's needed to do a basic version of this
already.

### Rework lighting for compatibility with glTF PBR material shading

We currently use our own ad-hoc method for representing normals and using them
in lighting calculations. glTF specifies that normals should be represented in
the tangent space at each vertex, so each vertex needs to have tangent, bitangent,
and normal vectors. We have implemented this properly according the the current
convention in the [Vulkan Grapher](https://github.com/seansovine/vulkan_grapher)
project. We could port the mesh generation and shader code from there to this
project.
