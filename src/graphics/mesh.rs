use crate::{bind_buffer, cstr, gen_attrib_pointers, Renderer, Shader, TextureHandle, Vertex, DEFAULT_SHADER};

use std::ops::{Index, IndexMut};
use std::{collections::HashMap, ptr};

use std::ffi::CString;

use gl::{*, types::GLsizei};
use glam::{Mat4, Quat, Vec3};

#[derive(PartialEq, Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,

    pub vao: u32,
    pub texture: u32,
    ebo: u32,
    vbo: u32,

    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub color: Vec3,

    pub shader: Shader,
    pub parent: Option<Box<Mesh>>,
    pub children: Vec<Box<Mesh>>,

    pub has_been_set_up: bool,
}

pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: &Vec<Vertex>, indices: &Vec<u32>) -> Self {
        let mesh = Mesh {
            vertices: vertices.to_vec(), indices: indices.to_vec(),
            vao: 0, vbo: 0, ebo: 0,
            position: Vec3::ZERO,
            rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            scale: Vec3::ONE,
            texture: 0,
            color: Vec3::ONE,
            shader: *DEFAULT_SHADER,
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

    pub fn setup_mesh(&mut self) {
        unsafe {
            GenVertexArrays(1, &mut self.vao);
            GenBuffers(1, &mut self.vbo);
            GenBuffers(1, &mut self.ebo);
    
            BindVertexArray(self.vao);
    
            bind_buffer!(ARRAY_BUFFER, self.vbo, self.vertices);
            bind_buffer!(ELEMENT_ARRAY_BUFFER, self.ebo, self.indices);
            gen_attrib_pointers!(Vertex, 0 => position: 3, 1 => color: 4, 2 => tex_coords: 2, 3 => normal: 3);
    
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            BindVertexArray(0);
        }
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
    pub fn add_mesh_from_vertices_and_indices(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>) -> Option<MeshHandle> {
        let handle = MeshHandle {id: self.meshes.len()};

        if self.meshes.contains_key(&handle) {
            println!("Mesh with handle {:?} already exists", handle);
            return None;
        }

        let mesh = Mesh::new(&vertices, &indices);
        
        self.meshes.insert(handle, mesh);

        Some(handle)
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> Option<MeshHandle> {
        let handle = MeshHandle {id: self.meshes.len()};

        if self.meshes.contains_key(&handle) {
            println!("Mesh with handle {:?} already exists", handle);
            return None;
        }

        self.meshes.insert(handle, mesh);
        Some(handle)
    }

    pub fn get_mesh_mut(&mut self, handle: MeshHandle) -> Option<&mut Mesh> {
        self.meshes.get_mut(&handle)
    }

    pub fn get_mesh(&self, handle: MeshHandle) -> Option<&Mesh> {
        self.meshes.get(&handle)
    }

    pub fn destroy_mesh(&mut self, handle: MeshHandle) {
        if self.meshes.remove(&handle).is_some() {

        } else {
            println!("Failed to remove mesh, or there was no mesh to remove");
        }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            DeleteVertexArrays(1, &self.vao);
            DeleteBuffers(1, &self.ebo);
            DeleteBuffers(1, &self.vbo);
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct MeshHandle {
    pub id: usize,
}

impl Index<MeshHandle> for HashMap<MeshHandle, Mesh> {
    type Output = Mesh;

    fn index(&self, handle: MeshHandle) -> &Self::Output {
        self.get(&handle).expect("No entry found for key")
    }
}

impl IndexMut<MeshHandle> for HashMap<MeshHandle, Mesh> {
    fn index_mut(&mut self, handle: MeshHandle) -> &mut Self::Output {
        self.get_mut(&handle).expect("No entry found for key")
    }
}