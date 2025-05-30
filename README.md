# WGPU Grapher

This is a 3D function grapher written in Rust using WGPU.

__Example__ $y = \text{sinc}(\sqrt{x^2 + y^2})$:

<p align="center" margin="20px">
	<img src="images/screenshot_1.png" alt="drawing" width="600" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

It has two main parts: A simple rendering engine to render a
vector of meshes with solid colored or textured primitives; and code to generate
the meshes for rendering a function graph.

The designs of the camera controller and main event
loop were inspired by the _Learn WGPU_ tutorial.

### Key controls:

| key     | action       |
| :------ | :------:     |
| `left`  | rotate left  |
| `right` | rotate right |
| `up`    | rotate up    |
| `down`  | rotate down  |
| `z`     | zoom in      |
| `x`     | zoom out     |
| `esc`   | exit         |

## 2D wave equation simulation

It now includes a finite-difference simulation of the 2D wave equation.

<p align="center" margin="20px">
	<img src="images/screenshot_wave_eqn.png" alt="drawing" width="600" style="padding-top: 15px; padding-bottom: 10px"/>
</p>

The idea for the random disturbances and energy damping in this simulation
came from [Beltoforion](https://beltoforion.de/en/recreational_mathematics/).
(His work is awesome; it's definitely worth checking out.)

You can now also render the wave equation simulation into a texture on a 2D canvas.

<p align="center" margin="20px">
	<img src="images/screenshot_wave_eqn_texture.png" alt="drawing" width="500" style="padding-top: 15px; padding-bottom: 10px"/>
</p>

This looks similar to Beltoforion's example referenced above.

## Image viewer

Now also includes an image viewer that renders an image from a file on a canvas
that can be zoomed and rotated, as in the other modes.

<p align="center" margin="20px">
	<img src="images/screenshot_image_viewer.png" alt="drawing" width="500" style="padding-top: 10px; padding-bottom: 10px"/>
</p>

(Sample image credit to Arjay Neyra on [Pexels](https://www.pexels.com/photo/spectacular-himalayan-mountain-valley-in-nepal-32225792/).)

In the future this will be used for some image processing and animated simulation applications.
