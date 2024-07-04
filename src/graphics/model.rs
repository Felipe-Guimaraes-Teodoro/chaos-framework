use std::{collections::HashMap, hash::Hash, ops::{Index, IndexMut}, path::Path};

use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4};
use russimp::{bone::{Bone, VertexWeight}, mesh::Mesh as rMesh, scene::Scene, Matrix4x4};
use tobj::LoadOptions;
use gl::types::GLuint;

use crate::{sBone, sMesh, sVertex, BoneInfo, Mesh, Renderer, Texture, Vertex, MAX_BONE_INFLUENCE};


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

    pub bone_info_map: HashMap<String, BoneInfo>,
    bone_counter: i32,
}

impl Model {
    pub fn new(path: &str) -> Self {
        let mut model = Model::default();
        model.load(path);

        model
    }

    pub fn empty() -> Self {
        Self {
            meshes: vec![],
            loaded_textures: vec![],
            bone_info_map: HashMap::new(),
            bone_counter: 0,
        }
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

    pub fn process_mesh(&mut self, scene: &Scene, id: usize) -> sMesh {
        let rmesh = &scene.meshes[id];

        let mut vertices = vec![];
        let indices: Vec<u32> = rmesh.faces.iter()
            .flat_map(|face| face.0.iter().copied())
            .collect();

        for i in 0..rmesh.vertices.len() {
            let mut vertex = sVertex::default();

            vertex.position = vec3(rmesh.vertices[i].x, rmesh.vertices[i].y, rmesh.vertices[i].z);
            vertex.normal = vec3(rmesh.normals[i].x, rmesh.normals[i].y, rmesh.normals[i].z);

            if let Some(Some(tex_coords)) = rmesh.texture_coords.get(0) {
                vertex.tex_coords = tex_coords.get(i).map_or(Vec2::ZERO, |tc| Vec2::new(tc.x, tc.y));
            } else {
                vertex.tex_coords = Vec2::ZERO;
            }

            self.set_vertex_bone_data_to_default(&mut vertex);

            vertices.push(vertex);
        }

        self.extract_bone_weight_for_vertices(&mut vertices, rmesh);

        sMesh::new(vertices, indices)
    }

    fn set_vertex_bone_data(vertex: &mut sVertex, id: i32, weight: f32) {
        for i in 0..MAX_BONE_INFLUENCE {
            if vertex.bone_ids[i] < 0 {
                vertex.weights[i] = weight;
                vertex.bone_ids[i] = id; 
                break;
            }
        }
    }

    fn extract_bone_weight_for_vertices(&mut self, vertices: &mut Vec<sVertex>, mesh: &rMesh) {
        for bone_index in 0..mesh.bones.len() {
            let mut bone_id = -1;
            let bone_name = mesh.bones[bone_index].name.clone();

            if !self.bone_info_map.contains_key(&bone_name) {
                let new_bone_info = BoneInfo {
                    id: self.bone_counter,
                    ofs: convert_russimp_mat_to_glam_mat(mesh.bones[bone_index].offset_matrix),
                };
                self.bone_info_map.insert(bone_name.clone(), new_bone_info);
                bone_id = self.bone_counter;
                self.bone_counter += 1;
            } else {
                bone_id = self.bone_info_map[&bone_name].id;
            }

            assert!(bone_id != -1);

            let weights = &mesh.bones[bone_index].weights;
            let num_weights = weights.len();

            for weight_index in 0..num_weights {
                let vertex_id = weights[weight_index].vertex_id;
                let weight = weights[weight_index].weight;
                assert!(vertex_id < vertices.len() as u32);
                Self::set_vertex_bone_data(&mut vertices[vertex_id as usize], bone_id, weight);
            }
        }
    }

    fn set_vertex_bone_data_to_default(&self, vertex: &mut sVertex) {
        for i in 0..MAX_BONE_INFLUENCE {
            vertex.bone_ids[i] = -1;
            vertex.weights[i] = 0.0;
        }
    }
}

pub fn convert_russimp_mat_to_glam_mat(mat: Matrix4x4) -> Mat4 {
    Mat4::from_cols_array(&[
        mat.a1, mat.a2, mat.a3, mat.a4,
        mat.b1, mat.b2, mat.b3, mat.b4,
        mat.c1, mat.c2, mat.c3, mat.c4,
        mat.d1, mat.d2, mat.d3, mat.d4,
    ])
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