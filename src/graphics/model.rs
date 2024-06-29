use std::{collections::HashMap, ops::{Index, IndexMut}, path::Path};

use glam::{vec2, vec3, vec4, Vec3, Vec4};
use tobj::LoadOptions;
use gl::types::GLuint;

use crate::{Mesh, Renderer, Texture, Vertex};


#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct ModelHandle {
    pub id: usize,
}

impl Index<ModelHandle> for HashMap<ModelHandle, Model> {
    type Output = Model;

    fn index(&self, handle: ModelHandle) -> &Self::Output {
        self.get(&handle).expect("No entry found for key")
    }
}

impl IndexMut<ModelHandle> for HashMap<ModelHandle, Model> {
    fn index_mut(&mut self, handle: ModelHandle) -> &mut Self::Output {
        self.get_mut(&handle).expect("No entry found for key")
    }
}

#[derive(Default)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub loaded_textures: Vec<GLuint>,
}

impl Model {
    pub fn new(path: &str) -> Self {
        let mut model = Model::default();
        model.load(path);

        model
    }

    pub fn extract_mesh(&mut self, path: &str, u: usize) -> Mesh {
        let path = Path::new(path);
        
        let load_options = LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        };

        let obj = tobj::load_obj(path, &load_options).expect("Failed to load OBJ file");
        let (models, _) = obj;

        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;
            let indices: Vec<u32> = mesh.indices.clone();

            let mut vertices = Vec::with_capacity(num_vertices);

            let (p, n, t, c) = (&mesh.positions, &mesh.normals, &mesh.texcoords, &mesh.vertex_color);

            for i in 0..num_vertices {
                let pos = vec3(p[i*3], p[i*3+1], p[i*3+2]);
                let tex_coords = vec2(t[i*2], t[i*2+1]);
                let normal = if n.len() >= (i + 1) * 3 {
                    vec3(n[i * 3], n[i * 3 + 1], n[i * 3 + 2])
                } else {
                    Vec3::ZERO
                };
                let color = if c.len() >= (i + 1) * 3 {
                    vec4(c[i * 3], c[i * 3 + 1], c[i * 3 + 2], 1.0)
                } else {
                    Vec4::ONE
                };
                vertices.push(
                    Vertex::new(pos, color, tex_coords, normal)
                );
            }

            
            self.meshes.push(Mesh::new(&vertices, &indices));
        }

        self.meshes[u].clone()
    }

    pub fn load(&mut self, path: &str) {
        let path = Path::new(path);

        let load_options = LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        };

        let obj = tobj::load_obj(path, &load_options).expect("Failed to load OBJ file");
        let (models, _) = obj;

        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;
            let indices: Vec<u32> = mesh.indices.clone();

            let mut vertices = Vec::with_capacity(num_vertices);

            let (p, n, t, c) = (&mesh.positions, &mesh.normals, &mesh.texcoords, &mesh.vertex_color);
            let mut last_normal = vec3(0.0, 0.0, 0.0);
            let mut last_tex_coord = vec2(0.0, 0.0);
            for i in 0..num_vertices {
                let pos = vec3(p[i*3], p[i*3+1], p[i*3+2]);
                // let tex_coords = vec2(t[i*2], t[i*2+1]);
                let tex_coords = if n.len() >= (i + 1) * 2 {
                    last_tex_coord = vec2(t[i * 2], t[i * 2 + 1]);
                    vec2(t[i * 2], t[i * 2 + 1])
                } else {
                    last_tex_coord
                };
                let normal = if n.len() >= (i + 1) * 3 {
                    last_normal = vec3(n[i * 3], n[i * 3 + 1], n[i * 3 + 2]);
                    vec3(n[i * 3], n[i * 3 + 1], n[i * 3 + 2])
                } else {
                    last_normal
                };
                let color = if c.len() >= (i + 1) * 3 {
                    vec4(c[i * 3], c[i * 3 + 1], c[i * 3 + 2], 1.0)
                } else {
                    Vec4::ONE
                };
                vertices.push(
                    Vertex::new(pos, color, tex_coords, normal)
                );
            }

            let mut final_mesh = Mesh::new(&vertices, &indices);

            for face in &mut final_mesh.indices.chunks_mut(6) {
                face.reverse();
            }

            final_mesh.setup_mesh();

            self.meshes.push(final_mesh);
        }
    }

    pub fn load_texture<'t>(&mut self, path: &'t str) -> Texture<'t> {
        let texture = Texture::Path(path);
        if let Texture::Loaded(id) = texture {
            self.loaded_textures.push(id);
        }

        texture
    }

    pub unsafe fn draw(&self) {
        for mesh in &self.meshes {
            mesh.draw();
        }
    } 
}


impl Renderer {
    pub fn add_model(&mut self, model: Model) -> Option<ModelHandle> {
        let handle = ModelHandle {id: self.models.len()};

        if self.models.contains_key(&handle) {
            println!("Model with handle {:?} already exists", handle);
            return None;
        }

        self.models.insert(handle, model);
        Some(handle)
    }

    pub fn destroy_model(&mut self, handle: ModelHandle) {
        self.models.remove(&handle);
    }
}

// idk if this is necessary since texture already implements drop (as well as mesh)
impl Drop for Model {
    fn drop(&mut self) {
        for texture_id in self.loaded_textures.iter() {
            unsafe {
                gl::DeleteTextures(1, texture_id);
            }
        }
    }
}