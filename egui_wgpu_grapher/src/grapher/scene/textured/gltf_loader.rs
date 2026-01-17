//! Read info from a glTF file using the document and buffers returned
//! by the `import` function of the glTF crate (its higher-level API).

use std::{cell::RefCell, path::Path};

use cgmath::{Matrix4, SquareMatrix};
use egui_wgpu::wgpu::{Device, Queue};
use gltf::{
    Document, Mesh, Node, Primitive, buffer::Data, image::Source, mesh::Mode, scene::Transform,
};

use crate::grapher::{
    matrix::MatrixUniform,
    pipeline::texture::{Image, TextureData},
    scene::{GpuVertex, textured::TexturedMeshData},
};

const DEFAULT_COLOR: [f32; 3] = [1.0, 0.0, 0.0];
const DEFAULT_COLOR_U8: [u8; 4] = [255, 0, 0, 255];

const DEV_LOGGING: bool = false;

// ------------------------
// Structure for glTF data.

pub struct RenderMesh {
    pub data: TexturedMeshData,
    pub matrix: MatrixUniform,
}

#[derive(Default)]
pub struct RenderScene {
    pub meshes: Vec<RenderMesh>,
}

// ------------
// glTF loader.

fn node_matrix(node: &Node) -> MatrixUniform {
    match node.transform() {
        Transform::Matrix { matrix } => matrix.into(),

        Transform::Decomposed {
            translation,
            rotation,
            scale,
        } => {
            let nontrivial = translation != [0.0_f32, 0.0_f32, 0.0_f32]
                || rotation != [0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32]
                || scale != [1.0_f32, 1.0_f32, 1.0_f32];

            if nontrivial {
                let t = cgmath::Matrix4::from_translation(translation.into());
                let r: cgmath::Matrix4<_> =
                    cgmath::Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2])
                        .into();

                // Scale matrix.
                let mut s: Matrix4<f32> = cgmath::Matrix4::identity();
                s[0][0] = scale[0];
                s[1][1] = scale[1];
                s[2][2] = scale[2];

                let transform: [[f32; 4]; 4] = (t * r * s).into();
                if DEV_LOGGING {
                    println!(">>> Node transform matrix:");
                    for row in transform {
                        println!(">>> {row:?}");
                    }
                }
                transform.into()
            } else {
                MatrixUniform::identity()
            }
        }
    }
}

pub struct GltfLoader<'a> {
    path: String,
    document: Document,
    buffer_data: Vec<Data>,

    device: &'a Device,
    queue: &'a Queue,

    // Could avoid RefCell, but it simplifies function signatures for now.
    render_scene: RefCell<RenderScene>,
}

impl<'a> GltfLoader<'a> {
    pub fn create(device: &'a Device, queue: &'a Queue, gltf_path: &str) -> GltfLoader<'a> {
        let (document, buffer_data, _images) = gltf::import(gltf_path).unwrap();
        GltfLoader {
            path: gltf_path.into(),
            document,
            buffer_data,
            device,
            queue,
            render_scene: Default::default(),
        }
    }
}

impl GltfLoader<'_> {
    pub fn traverse(self) -> RenderScene {
        let root_matrix = MatrixUniform::identity();
        for scene in self.document.scenes() {
            // Traverse root nodes of scene.
            for node in scene.nodes() {
                let matrix = root_matrix * node_matrix(&node);
                self.add_node(&node, 1, &matrix);
                self.traverse_children(&node, 2, &matrix);
            }
        }
        if DEV_LOGGING {
            println!(
                "# of meshes found: {}",
                self.render_scene.borrow().meshes.len()
            );
        }
        self.render_scene.into_inner()
    }

    fn traverse_children(&self, node: &Node, depth: usize, parent_matrix: &MatrixUniform) {
        for child in node.children() {
            let matrix = *parent_matrix * node_matrix(&child);
            self.add_node(&child, depth, &matrix);
            self.traverse_children(&child, depth + 1, &matrix);
        }
    }

    fn indent(depth: usize) {
        const INDENT: usize = 4;
        print!("{}", " ".repeat(depth * INDENT));
    }

    fn add_node(&self, node: &Node, depth: usize, matrix: &MatrixUniform) {
        if DEV_LOGGING {
            // Some logging.
            Self::log_node(node, depth);
        }
        if let Some(mesh) = node.mesh() {
            self.add_mesh(&mesh, depth + 1, matrix);
        }
    }

    fn log_node(node: &Node, depth: usize) {
        Self::indent(depth);
        println!(
            "Node {}: {}",
            node.index(),
            node.name().unwrap_or("<UNNAMED>")
        );
        match node.transform() {
            Transform::Matrix { matrix: _ } => {
                Self::indent(depth + 1);
                println!("Node has matrix.");
            }
            Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => {
                Self::indent(depth + 1);
                println!("Node has decomposed transformation.");

                let nontrivial = translation != [0.0_f32, 0.0_f32, 0.0_f32]
                    || rotation != [0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32]
                    || scale != [1.0_f32, 1.0_f32, 1.0_f32];

                if nontrivial {
                    Self::indent(depth + 2);
                    println!("Nontrivial translation: {translation:?}");
                    Self::indent(depth + 2);
                    println!("Rotation: {rotation:?}");
                    Self::indent(depth + 2);
                    println!("Scale: {scale:?}");
                }
            }
        }
        println!();
    }

    fn add_mesh(&self, mesh: &Mesh, depth: usize, matrix: &MatrixUniform) {
        if DEV_LOGGING {
            Self::indent(depth);
            println!("Node has mesh.");
        }

        let mut vertices = vec![];
        let mut indices = vec![];
        let mut texture = None;

        for primitive in mesh.primitives() {
            if primitive.mode() != Mode::Triangles {
                continue;
            }
            let reader = primitive.reader(|buff_idx| Some(&self.buffer_data[buff_idx.index()]));

            // Add position and normal coordinates.
            let iter = reader
                .read_positions()
                .unwrap()
                .zip(reader.read_normals().unwrap());
            for (position, normal) in iter {
                vertices.push(GpuVertex {
                    position,
                    color: DEFAULT_COLOR,
                    normal,
                    ..Default::default()
                });
            }

            // Add indices.
            indices = reader.read_indices().unwrap().into_u32().collect();

            // Read or create texture.
            let model_path = Path::new(&self.path).parent().unwrap();
            texture = read_texture(self.device, self.queue, &primitive, model_path)
                .unwrap_or_else(|()| {
                    let base_color = primitive
                        .material()
                        .pbr_metallic_roughness()
                        .base_color_factor();
                    let base_color = [
                        (255.0 * base_color[0]) as u8,
                        (255.0 * base_color[1]) as u8,
                        (255.0 * base_color[2]) as u8,
                        255,
                    ];
                    TextureData::solid_color_texture(&base_color, self.device, self.queue)
                })
                .into();

            // First set of texture coords is for vertex color.
            if let Some(iter) = reader.read_tex_coords(0) {
                vertices
                    .iter_mut()
                    .zip(iter.into_f32())
                    .for_each(|(vertex, tex_coords)| vertex.tex_coords = tex_coords);
            }
        }

        if DEV_LOGGING {
            println!();
        }

        let mut render_scene = self.render_scene.borrow_mut();
        render_scene.meshes.push(RenderMesh {
            data: TexturedMeshData {
                vertices,
                indices,
                texture: texture.unwrap(),
            },
            matrix: *matrix,
        });
    }
}

pub fn read_texture(
    device: &Device,
    queue: &Queue,
    primitive: &Primitive<'_>,
    model_dir: &Path,
) -> Result<TextureData, ()> {
    let pbr_metallic = primitive.material().pbr_metallic_roughness();

    if let Some(info) = pbr_metallic.base_color_texture() {
        let image_source = info.texture().source().source();
        match image_source {
            Source::Uri { uri, .. } => {
                let img_path = model_dir.join(Path::new(uri));
                let Ok(image) = Image::from_file(img_path.to_str().unwrap()) else {
                    return Err(());
                };
                let texture = TextureData::from_image(&image, device, queue);

                return Ok(texture);
            }
            Source::View { .. } => {
                println!("Warning: Buffer view texture will not be loaded.");
            }
        }
    }

    Err(())
}
