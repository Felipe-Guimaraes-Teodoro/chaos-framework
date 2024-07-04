use std::collections::HashMap;

use gl::{UseProgram, types::GLuint};

use crate::{sMesh, sMeshHandle, Camera, Light, LightHandle, Mesh, MeshHandle, Model, ModelHandle, TextureHandle, DEFAULT_SHADER, RUSSIMP_SHADER};

pub struct Renderer {
    pub meshes: HashMap<MeshHandle, Mesh>,
    pub animated_meshes: HashMap<sMeshHandle, sMesh>,
    pub lights: HashMap<LightHandle, Light>,
    pub textures: HashMap<TextureHandle, GLuint>,
    pub models: HashMap<ModelHandle, Model>,
    pub camera: Camera,
} 

impl Renderer {
    pub fn new() -> Self {
        let camera = Camera::new();
        Self {
            camera,
            meshes: HashMap::new(),
            animated_meshes: HashMap::new(),
            lights: HashMap::new(),
            textures: HashMap::new(),
            models: HashMap::new(),
        }
    }

    pub unsafe fn draw(&mut self) {
        DEFAULT_SHADER.use_shader();
        self.camera.send_uniforms(&DEFAULT_SHADER);
        self.send_light_uniforms(&DEFAULT_SHADER);
        UseProgram(0);

        RUSSIMP_SHADER.use_shader();
        self.camera.send_uniforms(&RUSSIMP_SHADER);
        self.send_light_uniforms(&RUSSIMP_SHADER);
        UseProgram(0);

        for mesh in self.meshes.values_mut() {
            if mesh.has_been_set_up == false {
                mesh.setup_mesh();
                mesh.has_been_set_up = true;
            }
            mesh.draw();
        }

        for model in self.models.values() {
            model.draw();
        }

        for animated_mesh in self.animated_meshes.values_mut() {
            if animated_mesh.has_been_set_up == false {
                animated_mesh.setup_mesh();
                animated_mesh.has_been_set_up = true;
            }
            animated_mesh.draw();
        }
    }

/*
    pub fn update(&mut self, el: &EventLoop) {
        for particle in self.particles.values_mut() {
            particle.update(&el);
        }
    }
*/
/* 
    pub unsafe fn draw(&self, el: &EventLoop) {
        INSTANCE_SHADER.use_shader();
        self.camera.send_uniforms(&INSTANCE_SHADER);
        UseProgram(0);

        PARTICLE_SHADER.use_shader();
        self.camera.send_uniforms(&PARTICLE_SHADER);
        UseProgram(0);

        DEFAULT_SHADER.use_shader();
        self.camera.send_uniforms(&DEFAULT_SHADER);
        UseProgram(0);

        LIGHT_SHADER.use_shader();
        self.camera.send_uniforms(&LIGHT_SHADER);
        self.send_light_uniforms(&LIGHT_SHADER);
        UseProgram(0);

        FULL_SHADER.use_shader();
        self.camera.send_uniforms(&FULL_SHADER);
        self.send_light_uniforms(&FULL_SHADER);
        UseProgram(0);


        for value in &self.instance_meshes {
            value.1.draw(&el);
        }
        
        for value in &self.meshes {
            value.1.draw();
        }

        for model in &self.models {
            model.1.draw();
        }

        for particle in &self.particles {
            particle.1.draw();
        }
    }
    */
}