use std::collections::HashMap;

use gl::types::GLuint;

use crate::{Camera, Light, LightHandle, Mesh, MeshHandle, Model, ModelHandle, SkeletalMesh, SkeletalMeshHandle, TextureHandle};

pub struct Renderer {
    pub meshes: HashMap<MeshHandle, Mesh>,
    pub lights: HashMap<LightHandle, Light>,
    pub textures: HashMap<TextureHandle, GLuint>,
    pub models: HashMap<ModelHandle, Model>,
    pub skeletal_meshes: HashMap<SkeletalMeshHandle, SkeletalMesh>,
    pub camera: Camera,
} 

impl Renderer {
    pub fn new() -> Self {
        let camera = Camera::new();

        Self {
            camera,
            meshes: HashMap::new(),
            lights: HashMap::new(),
            textures: HashMap::new(),
            models: HashMap::new(),
            skeletal_meshes: HashMap::new(),
        }
    }

    pub fn update(&mut self) {
        for mesh in self.meshes.values_mut() {
            if mesh.has_been_set_up == false {
                mesh.setup_mesh();
                mesh.has_been_set_up = true;
            }
        }

        for mesh in self.skeletal_meshes.values_mut() {
            if mesh.has_been_set_up == false {
                mesh.setup_mesh();
                mesh.has_been_set_up = true;
            }
        }
    }

    pub unsafe fn draw(&self) {
        // regular meshes
        for mesh in self.meshes.values() {
            mesh.draw(&self);
        }
        
        for model in self.models.values() {
            model.draw(&self);
        }
        
        // skeletal meshes
        for mesh in self.skeletal_meshes.values() {
            mesh.draw(&self);
        }
    }
}