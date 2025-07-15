# WGPU Grapher

This started out as an application to graph functions of the form `y = f(x,z)` in 3D using the
Rust `wgpu` crate, but is growing into a multi-purpose image and graphics application.

It now has a GUI built using `egui`. It is coming along, but is still a work in progress.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/graph.png?raw=true" alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

The `wgpu_grapher` crate in this repository has an older version of the app with
some features that haven't been ported to the GUI version. Some of those are discussed
[here](./GrapherCaps.md).

In the future I plan to add an expression parsing library so a function to graph can be
entered in the UI, and I plan to expand the UI for adjusting and interacting with the graph.

## glTF model viewer

There is a model-viewer mode that loads and renders a 3D model in the [glTF](https://kcoley.github.io/glTF/specification/2.0/figures/gltfOverview-2.0.0a.png)
format. It currently only supports the base color textures, and importantly it doesn't
yet handle sub-mesh matrices, so some models will not render correctly. I plan to add support
for those and also to start implementing more of the PBR rendering supported by glTF, and later
maybe even support for articulated models.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/model.png?raw=true" alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

*Model from MG Rips on [SketchFab](https://sketchfab.com/3d-models/secret-of-the-mimic-mimic-43d8bb26c79148958f46eb1d10e76667).*

## Image viewer

There is also an image-viewer mode that loads and displays an image file. In the future I
may incorporate some image processing features into this part.

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

## License and credit

This software is licensed under the MIT license. For the `egui` integration I started
with the `winit` + `egui` + `wgpu` template [here](https://github.com/kaphula/winit-egui-wgpu-template),
which is also licensed under the MIT license. In setting up the `wgpu` code I
followed the [Learn WGPU](https://sotrh.github.io/learn-wgpu/)
tutorial, and was influenced by the design of the code there. For some of the
graphics concepts and implementations I learned from the [Learn OpenGL](https://learnopengl.com/)
tutorial.
