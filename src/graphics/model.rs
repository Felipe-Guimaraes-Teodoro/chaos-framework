use std::{cmp::Ordering, collections::HashMap, hash::Hash, ops::{Index, IndexMut}, path::Path};

use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4};
use russimp::{bone::{Bone, VertexWeight}, mesh::Mesh as rMesh, property::{Property, PropertyStore}, scene::{PostProcess, Scene}, Matrix4x4};
use tobj::LoadOptions;
use gl::types::GLuint;

use crate::{Mesh, Renderer, SkeletalMesh, SkeletalVertex, Texture, Vertex, MAX_BONE_INFLUENCE};


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
            bone_counter: 0,
        }
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

    /* TODO: function that takes in an empty mesh and loads its vertex data with russimp as well as bones accordingly */

    pub fn load_russimp<'t>(path: &'t str) -> (SkeletalMesh, Scene) {
        let scene = Scene::from_file(
            path, 
            vec![
                PostProcess::Triangulate,

            ],
        ).unwrap();

        let russimp_mesh = &scene.meshes[0];
        
        let positions = russimp_mesh.vertices.iter().map(|v| {
            return vec3(v.x, v.y, v.z);
        }).collect::<Vec<Vec3>>();

        let normals = russimp_mesh.normals.iter().map(|n| {
            return vec3(n.x, n.y, n.z);
        }).collect::<Vec<Vec3>>();

        let indices = russimp_mesh.faces.iter().map(|f| {
            return f.0.clone();
        }).flatten().collect::<Vec<u32>>();

        let tex_coords = russimp_mesh.texture_coords[0].clone().unwrap().iter().map(|t| {
            return vec2(t.x, t.y);
        }).collect::<Vec<Vec2>>();

        let bones = &russimp_mesh.bones;

        let (bone_ids, weights) = Self::collect_bone_data(bones);

        let mut vertices = vec![];

        for i in 0..positions.len() {
            vertices.push(SkeletalVertex {
                position: positions[i],
                normal: normals[i],
                tex_coords: tex_coords[i],
                bone_ids: bone_ids[i],
                weights: weights[i],
            });
        }
        
        (SkeletalMesh::new(&vertices, &indices), scene)
    }

    fn collect_bone_data(bones: &Vec<Bone>) -> (Vec<[i32; MAX_BONE_INFLUENCE]>, Vec<[f32; MAX_BONE_INFLUENCE]>) {
        let num_vertices = bones.iter().flat_map(|bone| bone.weights.iter()).map(|w| w.vertex_id).max().unwrap_or(0) + 1;
    
        let mut bone_ids = vec![[0; MAX_BONE_INFLUENCE]; num_vertices as usize];
        let mut weights = vec![[0.0; MAX_BONE_INFLUENCE]; num_vertices as usize];
    
        let mut vertex_influences: Vec<Vec<(i32, f32)>> = vec![Vec::new(); num_vertices as usize];
    
        for (bone_id, bone) in bones.iter().enumerate() {
            for weight in &bone.weights {
                vertex_influences[weight.vertex_id as usize].push((bone_id as i32, weight.weight));
            }
        }
    
        for (vertex_id, influences) in vertex_influences.iter_mut().enumerate() {
            influences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    
            for (i, &(bone_id, weight)) in influences.iter().take(MAX_BONE_INFLUENCE).enumerate() {
                bone_ids[vertex_id][i] = bone_id;
                weights[vertex_id][i] = weight;
            }
        }
    
        (bone_ids, weights)
    }
}

pub fn convert_russimp_mat_to_glam_mat(mat: Matrix4x4) -> Mat4 {
    Mat4::from_cols_array(&[
        mat.a1, mat.b1, mat.c1, mat.d1,
        mat.a2, mat.b2, mat.c2, mat.d2,
        mat.a3, mat.b3, mat.c3, mat.d3,
        mat.a4, mat.b4, mat.c4, mat.d4,
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