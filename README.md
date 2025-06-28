# WGPU Grapher

This is an application to graph functions of the form `y = f(x,z)` in 3D using the
Rust `wgpu` crate. It now has a (work in progress) GUI built using `egui`.

<p align="center" margin="20px">
	<img src="https://github.com/seansovine/page_images/blob/main/screenshots/wgpu_grapher_2025-06-28.png?raw=true" alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

The `wgpu_grapher` crate in this repository has an older version of the app with
some features that haven't been ported to the GUI version. Some of those are discussed
[here](./GrapherCaps.md).

In the future I plan to add an expression parsing library so a function to graph can be
entered in the UI, and I plan to expand the UI for adjusting and interacting with the graph.

## Key controls

| key     | action       |
| :------ | :------:     |
| `left`  | rotate left  |
| `right` | rotate right |
| `up`    | rotate up    |
| `down`  | rotate down  |
| `z`     | zoom in      |
| `x`     | zoom out     |
| `esc`   | exit         |

## License and credit

This software is licensed under the MIT license. For the `egui` integration I started
with the `winit` + `egui` + `wgpu` template [here](https://github.com/kaphula/winit-egui-wgpu-template),
which is also licensed under the MIT license. In setting up the `wgpu` code I
followed the [Learn WGPU](https://sotrh.github.io/learn-wgpu/)
tutorial, and was influenced by the design of the code there. For some of the
graphics concepts and implementations I learned from the [Learn OpenGL](https://learnopengl.com/)
tutorial.
