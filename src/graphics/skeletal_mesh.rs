use std::mem::size_of;

use glam::{Vec2, Vec3};

use gl::*;

use crate::{bind_buffer, gen_attrib_pointers, Mesh, RUSSIMP_SHADER};

pub const MAX_BONE_INFLUENCE: usize = 4;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SkeletalVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec2,
    // pub tangent: Vec3,
    // pub bitangent: Vec3,
    pub bone_ids: [i32; MAX_BONE_INFLUENCE],
    pub weights: [f32; MAX_BONE_INFLUENCE],
}

impl Default for SkeletalVertex {
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

use crate::{cstr, Renderer, Shader, TextureHandle};

use std::ops::{Index, IndexMut};
use std::{collections::HashMap, ptr};

use std::ffi::CString;

use gl::types::GLsizei;
use glam::{Mat4, Quat};

#[derive(PartialEq, Debug, Clone)]
pub struct SkeletalMesh {
    pub vertices: Vec<SkeletalVertex>,
    pub indices: Vec<u32>,

    pub vao: u32,
    pub texture: u32,
    pub ebo: u32,
    pub vbo: u32,

    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub color: Vec3,

    pub shader: Shader,
    pub parent: Option<Box<Mesh>>,
    pub children: Vec<Box<Mesh>>,

    pub has_been_set_up: bool,
}

impl SkeletalMesh {
    pub fn new(vertices: &Vec<SkeletalVertex>, indices: &Vec<u32>) -> Self {
        let mesh = Self {
            vertices: vertices.to_vec(), indices: indices.to_vec(),
            vao: 0, vbo: 0, ebo: 0,
            position: Vec3::ZERO,
            rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            scale: Vec3::ONE,
            texture: 0,
            color: Vec3::ONE,
            shader: *RUSSIMP_SHADER,
            parent: None,
            children: Vec::new(),
            has_been_set_up: false,
        };

        mesh
    }
    pub fn set_texture(&mut self, texture_handle: TextureHandle, renderer: &Renderer) {
        self.texture = renderer.textures[&texture_handle];
    }

    /* 
    pub fn to_instance(&mut self, data: Vec<InstanceData>, n: usize) -> InstanceMesh {
        let mut new_mesh = InstanceMesh::new(&self.vertices, &self.indices, n);

        new_mesh.instance_data = data;

        unsafe { new_mesh.setup_mesh() };

        drop(self);

        new_mesh
    }

    pub fn set_parent(&mut self, parent: Mesh){
        self.parent = Some(Box::new(parent));
    }

    pub fn add_child(&mut self, mut child: Mesh){
        child.set_parent(self.clone());
        self.children.push(Box::new(child));
    }
    */

    pub fn set_color(&mut self, color: Vec3){
        self.color = color;
    }

    pub fn set_position(&mut self, position: Vec3){
        self.position = position;
        for child in self.children.as_mut_slice(){
            child.set_position(position + child.position)
        }
    }

    pub fn add_position(&mut self, position: Vec3){
        self.position += position;
        for child in self.children.as_mut_slice(){
            child.add_position(position)
        }
    }

    pub fn set_scale(&mut self, scale: Vec3){
        self.scale = scale;
        for child in self.children.as_mut_slice(){
            child.set_scale(scale);
        }
    }

    pub fn scale(&mut self, scale: Vec3){
        self.scale *= scale;
        for child in self.children.as_mut_slice(){
            child.scale(scale);
        }
    }

    pub fn set_rotation(&mut self, rotation: Quat){
        self.rotation = rotation;
        for child in self.children.as_mut_slice(){
            child.set_rotation(rotation);
        }
    }

    pub fn rotate(&mut self, rotation: Quat){
        self.rotation = self.rotation + rotation;
        for child in self.children.as_mut_slice(){
            child.rotate(rotation);
        }
    }


    pub unsafe fn setup_mesh(&mut self) {
        let size = size_of::<SkeletalVertex>() as GLsizei;            
        GenVertexArrays(1, &mut self.vao);
        GenBuffers(1, &mut self.vbo);
        GenBuffers(1, &mut self.ebo);

        BindVertexArray(self.vao);

        bind_buffer!(ARRAY_BUFFER, self.vbo, self.vertices);
        bind_buffer!(ELEMENT_ARRAY_BUFFER, self.ebo, self.indices);
        gen_attrib_pointers!(SkeletalVertex, 0 => position: 3, 1 => normal: 3, 2 => tex_coords: 2);
        // now generate the other attrib pointers 

        // ids
        EnableVertexAttribArray(3);
        let offset_bone = &((*std::ptr::null::<SkeletalVertex>()).bone_ids) as *const _ as *const std::ffi::c_void;
        VertexAttribIPointer(3, 4, INT, size, offset_bone);

        // weights (we could generate this using the macro, but i prefer this)
        EnableVertexAttribArray(4);
        let offset_weight = &((*std::ptr::null::<SkeletalVertex>()).weights) as *const _ as *const std::ffi::c_void;
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

impl Renderer {
    pub fn add_skeletal_mesh_from_vertices_and_indices(&mut self, vertices: Vec<SkeletalVertex>, indices: Vec<u32>) -> Option<SkeletalMeshHandle> {
        let handle = SkeletalMeshHandle {id: self.meshes.len()};

        if self.skeletal_meshes.contains_key(&handle) {
            println!("Skeletal mesh with handle {:?} already exists", handle);
            return None;
        }

        let mesh = SkeletalMesh::new(&vertices, &indices);
        
        self.skeletal_meshes.insert(handle, mesh);

        Some(handle)
    }

    pub fn add_skeletal_mesh(&mut self, mesh: SkeletalMesh) -> Option<SkeletalMeshHandle> {
        let handle = SkeletalMeshHandle {id: self.skeletal_meshes.len()};

        if self.skeletal_meshes.contains_key(&handle) {
            println!("Skeletal mesh with handle {:?} already exists", handle);
            return None;
        }

        self.skeletal_meshes.insert(handle, mesh);
        Some(handle)
    }

    pub fn get_skeletal_mesh_mut(&mut self, handle: SkeletalMeshHandle) -> Option<&mut SkeletalMesh> {
        self.skeletal_meshes.get_mut(&handle)
    }

    pub fn get_skeletal_mesh(&self, handle: SkeletalMeshHandle) -> Option<&SkeletalMesh> {
        self.skeletal_meshes.get(&handle)
    }

    pub fn destroy_skeletal_mesh(&mut self, handle: SkeletalMeshHandle) {
        if self.skeletal_meshes.remove(&handle).is_some() {

        } else {
            println!("Failed to remove mesh, or there was no mesh to remove");
        }
    }
}

impl Drop for SkeletalMesh {
    fn drop(&mut self) {
        unsafe {
            DeleteVertexArrays(1, &self.vao);
            DeleteBuffers(1, &self.ebo);
            DeleteBuffers(1, &self.vbo);
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct SkeletalMeshHandle {
    pub id: usize,
}

impl Index<SkeletalMeshHandle> for HashMap<SkeletalMeshHandle, Mesh> {
    type Output = Mesh;

    fn index(&self, handle: SkeletalMeshHandle) -> &Self::Output {
        self.get(&handle).expect("No entry found for key")
    }
}

impl IndexMut<SkeletalMeshHandle> for HashMap<SkeletalMeshHandle, Mesh> {
    fn index_mut(&mut self, handle: SkeletalMeshHandle) -> &mut Self::Output {
        self.get_mut(&handle).expect("No entry found for key")
    }
}
