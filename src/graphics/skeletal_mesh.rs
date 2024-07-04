use std::{collections::HashMap, mem::size_of, ops::{Index, IndexMut}, path::Path, ptr};

use gl::{*, types::*};
use glam::{Mat4, Quat, Vec2, Vec3};
use image::{DynamicImage, ExtendedColorType, RgbaImage};
use russimp::{bone::{Bone, VertexWeight}, material::{DataContent, TextureType}, mesh::Mesh, property::{Property, PropertyStore}, scene::{PostProcess, Scene}};
use std::ffi::CString;

use crate::{bind_buffer, cstr, gen_attrib_pointers, Animation, Animator, Model, Renderer, Shader, TextureHandle, RUSSIMP_SHADER};

pub const MAX_BONE_INFLUENCE: usize = 4;

#[derive(Copy, Clone)]
pub struct sVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec2,
    // pub tangent: Vec3,
    // pub bitangent: Vec3,
    pub bone_ids: [i32; MAX_BONE_INFLUENCE],
    pub weights: [f32; MAX_BONE_INFLUENCE],
}

impl Default for sVertex {
    fn default() -> Self {
        Self { 
            position: Default::default(), 
            normal: Default::default(), 
            tex_coords: Default::default(), 
            // tangent: Default::default(), 
            // bitangent: Default::default(), 
            bone_ids: [-1; MAX_BONE_INFLUENCE], 
            weights: [0.0; MAX_BONE_INFLUENCE]
        }
    }
}



pub struct sMesh {
    pub vertices: Vec<sVertex>,
    pub indices: Vec<u32>,
    
    pub vao: u32,
    pub texture: u32,
    ebo: u32,
    vbo: u32,

    pub bones: Vec<Bone>,

    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub color: Vec3,

    pub shader: Shader,
    pub parent: Option<Box<Mesh>>,
    pub children: Vec<Box<Mesh>>,
    
    pub has_been_set_up: bool,
}

impl sMesh {
    pub fn new(vertices: Vec<sVertex>, indices: Vec<u32>) -> Self {
        let mesh = sMesh {
            vertices: vertices.to_vec(), indices: indices.to_vec(),
            vao: 0, vbo: 0, ebo: 0,
            position: Vec3::ZERO,
            rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            scale: Vec3::ONE,
            texture: 0,
            color: Vec3::ONE,
            shader: *RUSSIMP_SHADER,
            parent: None,
            bones: vec![],
            children: Vec::new(),
            has_been_set_up: false,
        };

        mesh
    }

    pub fn set_texture(&mut self, texture_handle: TextureHandle, renderer: &Renderer) {
        self.texture = renderer.textures[&texture_handle];
    }

    pub unsafe fn setup_mesh(&mut self) {
        let size = size_of::<sVertex>() as GLsizei;            
        GenVertexArrays(1, &mut self.vao);
        GenBuffers(1, &mut self.vbo);
        GenBuffers(1, &mut self.ebo);

        BindVertexArray(self.vao);

        bind_buffer!(ARRAY_BUFFER, self.vbo, self.vertices);
        bind_buffer!(ELEMENT_ARRAY_BUFFER, self.ebo, self.indices);
        gen_attrib_pointers!(sVertex, 0 => position: 3, 1 => normal: 3, 2 => tex_coords: 2);
        // now generate the other attrib pointers 

        // ids
        EnableVertexAttribArray(3);
        let offset_bone = &((*std::ptr::null::<sVertex>()).bone_ids) as *const _ as *const std::ffi::c_void;
        VertexAttribIPointer(3, 4, INT, size, offset_bone);

        // weights (we could generate this using the macro, but i prefer this)
        EnableVertexAttribArray(4);
        let offset_weight = &((*std::ptr::null::<sVertex>()).weights) as *const _ as *const std::ffi::c_void;
        VertexAttribPointer(4, 4, FLOAT, FALSE, size, offset_weight);

        gl::BindTexture(gl::TEXTURE_2D, self.texture);

        BindVertexArray(0);
    }

    pub unsafe fn draw(&self) {
        let model_matrix = 
            Mat4::from_translation(self.position) *
            Mat4::from_quat(self.rotation) *
            Mat4::from_scale(self.scale);

        BindVertexArray(self.vao);
        self.shader.use_shader();
        
        if self.texture != 0 {
            unsafe {
                self.shader.uniform_1i(cstr!("has_texture"), 1);
            }
        } else {
            unsafe {
                self.shader.uniform_1i(cstr!("has_texture"), 0);
            }
        }

        // Set uniforms and draw
        self.shader.uniform_mat4fv(cstr!("model"), &model_matrix.to_cols_array());
        self.shader.uniform_vec3f(cstr!("pos"), &self.position);
        self.shader.uniform_vec3f(cstr!("color"), &self.color);
        
        BindTexture(TEXTURE_2D, self.texture);

        DrawElements(TRIANGLES, self.indices.len() as i32, UNSIGNED_INT, ptr::null());

        BindVertexArray(0);
        UseProgram(0);
    }
}

// i could use russimp's Bone struct, but i wanna use glam's mat4 so here's a redefinition!
pub struct sBone {
    pub weights: Vec<VertexWeight>,
    pub name: String,
    pub offset_matrix: Mat4,
}

#[derive(Copy, Clone)]
pub struct BoneInfo {
    pub id: i32,
    pub ofs: Mat4,
}


#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct sMeshHandle {
    pub id: usize,
}

impl Index<sMeshHandle> for HashMap<sMeshHandle, sMesh> {
    type Output = sMesh;

    fn index(&self, handle: sMeshHandle) -> &Self::Output {
        self.get(&handle).expect("No entry found for key")
    }
}

impl IndexMut<sMeshHandle> for HashMap<sMeshHandle, sMesh> {
    fn index_mut(&mut self, handle: sMeshHandle) -> &mut Self::Output {
        self.get_mut(&handle).expect("No entry found for key")
    }
}

impl Renderer {
    pub fn add_animated_mesh(&mut self, smesh: sMesh) -> Option<sMeshHandle> {
        let handle = sMeshHandle {id: self.meshes.len()};

        if self.animated_meshes.contains_key(&handle) {
            println!("Mesh with handle {:?} already exists", handle);
            return None;
        }

        self.animated_meshes.insert(handle, smesh);
        Some(handle)
    }

    pub fn destroy_animated_mesh(&mut self, handle: sMeshHandle) {
        if self.animated_meshes.remove(&handle).is_some() {

        } else {
            println!("Failed to remove animated mesh, or there was no animated mesh to remove");
        }
    }
}

impl Drop for sMesh {
    fn drop(&mut self) {
        unsafe {
            DeleteVertexArrays(1, &self.vao);
            DeleteBuffers(1, &self.ebo);
            DeleteBuffers(1, &self.vbo);
        }
    }
}

pub fn load_scene(path: &str) -> Scene {
    let props: PropertyStore = PropertyStore::default();
    
    let scene = Scene::from_file_with_props(
        path,
        vec![
            // PostProcess::OptimizeMeshes, // hell yeah
            PostProcess::Triangulate,
            PostProcess::GenerateSmoothNormals,
            // PostProcess::FlipUVs,
            PostProcess::FlipWindingOrder,
            PostProcess::JoinIdenticalVertices,
            PostProcess::OptimizeGraph,
        ],
        &props,
    )
    .unwrap();

    scene
}

pub fn test() -> (sMesh, Animator) {
    let mut scene = load_scene("assets/models/untgaegeaitled.fbx");

    let mut model = Model::empty();

    let mut mesh = model.process_mesh(&mut scene, 0);
    mesh.color = Vec3::ONE;
    let animation = Animation::new("assets/models/untgaegeaitled.fbx", &mut model);
    let animator = Animator::new(animation);

    mesh.scale = Vec3::ONE;

    (mesh, animator)
}

