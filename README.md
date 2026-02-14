# Wgpu Grapher

This is a work-in-progress application to graph functions of the form `y = f(x,z)` in 3D using the
Rust Wgpu graphics API. It has a simple GUI built using egui, and also has a basic glTF model viewer
and an image viewer mode.

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

_Model from official glTF sample collection._

## Image viewer

There is also an image viewer mode that loads and displays an image file. In the future I
would like to add some image processing features to this part.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/image.png?raw=true" alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

_Image from Tom Swinnen on [Pexels](https://www.pexels.com/photo/seashore-under-blue-sky-and-white-clouds-view-946351/)._

## GPU wave equation simulation

The `gpu_finite_difference` crate has a finite-difference wave equation solver implemented on the
GPU using a Wgpu compute shader. The basic version of this has been integrated into the "solver"
mode of the Grapher.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/wave_eqn.gif?raw=true" alt="drawing" width="400" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

Eventually I want to merge this into the graph mode, so that it will use the user's input function as
an initial condition and update the graph with each timestep of the wave equation solution. There
is a version of this in the old `wgpu_grapher` crate, but that version does the solving on the CPU
and does a lot of work to copy the vertex data to the GPU for rendering. This version will be much
more efficient.

## Mouse controls

| Input                    | Action          |
| ------------------------ | --------------- |
| `click + drag`           | rotate graph    |
| `control + click + drag` | translate graph |
| `mouse wheel`            | zoom graph      |

## Key controls

| key     |    action    | key            |     action      |
| :------ | :----------: | :------------- | :-------------: |
| `left`  | rotate left  | `t`            |  translate up   |
| `right` | rotate right | `g`            | translate down  |
| `up`    |  rotate up   | `f`            | translate left  |
| `down`  | rotate down  | `h`            | translate right |
| `z`     |   zoom in    | `esc`          |      exit       |
| `x`     |   zoom out   | `shift` + \_\_ | increase speed  |

## Licenses and credits

For the egui integration I started with
[this](https://github.com/kaphula/winit-egui-wgpu-template)
Winit + egui + Wgpu template, which is released under the MIT license.
To learn the Wgpu API, I started with the
[Learn Wgpu](https://sotrh.github.io/learn-wgpu/)
tutorial, and was influenced by the design of the example code there. For many
graphics concepts and implementations I learned from the awesome
[Learn OpenGL](https://learnopengl.com/)
tutorial. Also, see my [thoughts on AI as a tool](./docs/AI.md).

This software is released under the MIT license.
