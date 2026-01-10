# Developer Notes

## Known issues and TODOs

### Parameter input bugs

The behavior of the graph relative to GUI parameter inputs is very buggy.
Likely we need to rework the way these values are stored and used into
a more sane structure.

### Shadow mapping bugs

Shadow mapping for the floor mesh doesn't seem to be working correctly.

To help debug this and other 3D rendering issues we will add some code to
optionally render scene objects for light position, coordinate axes, etc.

### Lighting edge cases

There seem to be some cases at creases or joints in the mesh where the
lighting looks strange. This could possibly be an issue of needing
a `max` or a `floor` function at certain places in the shader calculations.

_Example:_

+ Light position: `[3.0, 4.0, 0.0]`
+ Function: `5.0*e^(5.0*(-(x-4.5)^2 - (z-3)^2))`
+ x-shift: 0.0
+ y-shift: 0.0
+ z-shift: 0.0

From this example there may be two issues:

+ Our coordinate transformations are applied inconsistently.

There is almost surely a coordinate consistency issue here.

+ There is a problem edge case when the light is inside the object mesh.

TODO: Go through transformations and make them all consistent and add lighting debug code.

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
