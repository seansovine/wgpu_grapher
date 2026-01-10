# WGPU Grapher

This is mainly an application to graph functions of the form `y = f(x,z)` in 3D using the
Rust `wgpu` graphics API. I'm also using it as a playground for experimenting with
ideas and techniques in computer graphics and image processing.

It has a simple GUI built using `egui`. In graph mode the user can enter a mathematical expression
in the "Function" window, and if the expression is valid a graph for that function will be generated.
The mathematical expression parsing and evaluation is handled by the
[meval](https://docs.rs/meval/latest/meval/) crate.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/shadow_mapping_geometry_2026-01-10.png?raw=true" alt="drawing" width="700" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

_Lighting:_

As you can see in the example, the renderer implements Phong lighting and basic shadow mapping.

## Project status

This has been a learning platform and is a work in progress. I'm recently coming back to it
after working on other things for a while. There are a few known bugs that need worked out, and
I plan to rework some of the code architecture now that the application has grown more complex.
I hope to get around to cleaning these things up in the near future.

See [Developer Notes](docs/DevNotes.md) for some more details on these and other known issues.

_Older version:_

The `wgpu_grapher` crate in this repository has an older version of the app with
some features that haven't been ported to the GUI version. Some of those are discussed
[here](./GrapherCaps.md).

## glTF model viewer

There is a model viewer mode that loads and renders a scene in the [glTF](https://kcoley.github.io/glTF/specification/2.0/figures/gltfOverview-2.0.0a.png)
format. It currently only supports base color textures (no normal mapping, etc.), and importantly it doesn't
yet handle submesh transformations, so some models will not render correctly. I plan to add support
for those in the near future. I also plan to add support for the glTF PBR materials shading model
and later maybe some more of the other features supported by glTF.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/model.png?raw=true" alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

*Model from MG Rips on [SketchFab](https://sketchfab.com/3d-models/secret-of-the-mimic-mimic-43d8bb26c79148958f46eb1d10e76667).*

## Image viewer

There is also an image viewer mode that loads and displays an image file. In the future I
would like to add some image processing features to this part.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/image.png?raw=true" alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

*Image from Tom Swinnen on [Pexels](https://www.pexels.com/photo/seashore-under-blue-sky-and-white-clouds-view-946351/).*

## Key controls

| key     | action       | key     | action       |
| :------ | :------:     | :------ | :------:     |
| `left`  | rotate left  | `t` | translate up |
| `right` | rotate right | `g` | translate down |
| `up`    | rotate up    | `f` | translate left |
| `down`  | rotate down  | `h` | translate right |
| `z`     | zoom in      | `esc` | exit |
| `x`     | zoom out     | `shift` + \_\_ | increase speed |

## Licenses and credits

For the `egui` integration I started with
[this](https://github.com/kaphula/winit-egui-wgpu-template)
`winit` + `egui` + `wgpu` template , which is released under the MIT license.
To learn the `wgpu` API I started with the
[Learn WGPU](https://sotrh.github.io/learn-wgpu/)
tutorial, and was influenced by the design of the example code there. For many
graphics concepts and implementations I learned from the awesome
[Learn OpenGL](https://learnopengl.com/)
tutorial.

This software is released under the MIT license.
