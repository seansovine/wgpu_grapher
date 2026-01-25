# Wgpu Grapher

This is a work-in-progress application to graph functions of the form `y = f(x,z)` in 3D using the
Rust Wgpu graphics API, that also has basic glTF model viewer and image viewer functions.
It has a simple GUI built using egui.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/sine_spines_shadow.png?raw=true" alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

The renderer implements Phong lighting and basic shadow mapping. Mathematical expression
parsing and evaluation are handled by the [meval](https://docs.rs/meval/latest/meval/) crate.
The current version of this project is in the `egui_wgpu_grapher` crate in the folder with the same name.

The `wgpu_grapher` crate in this repository has an older version of the app with
some features that haven't been ported to the GUI version. Some of those are discussed
[here](./GrapherCaps.md).

## glTF viewer

The model viewer mode loads and renders a scene from a file in the [glTF](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html)
format. It currently only supports base color textures (no normal mapping, etc. yet),
and there are some things that are still works in progress. Hopefully we'll get to those soon.
Eventually I hope to add support for glTF PBR materials shading, and maybe some of the other
features supported by glTF.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/model_2.png?raw=true" alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

*Model from official glTF sample collection.*

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

For the egui integration I started with
[this](https://github.com/kaphula/winit-egui-wgpu-template)
Winit + egui + Wgpu template, which is released under the MIT license.
To learn the Wgpu API, I started with the
[Learn Wgpu](https://sotrh.github.io/learn-wgpu/)
tutorial, and was influenced by the design of the example code there. For many
graphics concepts and implementations I learned from the awesome
[Learn OpenGL](https://learnopengl.com/)
tutorial.

This software is released under the MIT license.
