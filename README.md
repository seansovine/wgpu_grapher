# Wgpu Grapher

This is a 3D function grapher built using the Rust Wgpu graphics API. It has a simple GUI built using egui,
and also has a basic glTF model viewer, an image viewer, and a GPU-accelerated wave equation solver.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/sine_spines_shadow.png?raw=true"
		alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

The renderer implements Phong lighting and basic shadow mapping. Mathematical expression
parsing and evaluation are handled by the [meval](https://docs.rs/meval/latest/meval/) crate. These
[notes](docs/DevNotes.md) have some interesting example functions and discuss some ideas that
have influenced the implementation.

The current version of this project is in the `egui_wgpu_grapher` crate. The `wgpu_grapher` crate has an older version
of the app with some features that haven't been ported to the GUI version. Some of those are discussed [here](./GrapherCaps.md).

## glTF viewer

The model viewer mode loads and renders a scene from a file in the [glTF](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html)
format. There are some glTF features that are not yet implemented or are works in progress. Eventually I hope to add support
for those, including glTF PBR materials shading.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/gltf_chess_board_shadow_debug.png?raw=true"
		alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

_Model from official glTF sample collection._

## Image viewer

There is also an image viewer mode that loads and displays an image file. In the future I
plan to add some image processing features to this.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/image.png?raw=true"
		alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

_Image from Tom Swinnen on [Pexels](https://www.pexels.com/photo/seashore-under-blue-sky-and-white-clouds-view-946351/)._

## GPU wave equation simulation

The solver mode of has a finite-difference wave equation solver implemented on the
GPU using a Wgpu compute shader. The `gpu_finite_difference` crate has a standalone version of the solver.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher/wave_eqn.gif?raw=true"
		alt="drawing" width="400" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

Eventually I plan to allow using the user's input function as an initial condition for the solver. There is a version of this in the
old `wgpu_grapher` crate, but that version runs the solver on the CPU and does a lot of work to copy the vertex data to the GPU.

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

For the egui integration I started with [this](https://github.com/kaphula/winit-egui-wgpu-template)
Winit + egui + Wgpu template, which is released under the MIT license. Meval also has the MIT license.
And like most Rust developers, I've depended heavily on the open source crate ecosystem.
To learn the Wgpu API, I started with the [Learn Wgpu](https://sotrh.github.io/learn-wgpu/)
tutorial, and was influenced by the design of the example code there. For many
graphics concepts and implementations I learned from the excellent [Learn OpenGL](https://learnopengl.com/)
tutorial. Also, see my [thoughts on AI as a tool](./docs/AI.md).

This software is released under the MIT license.
