use std::{collections::HashMap, ffi::CString, ops::{Index, IndexMut}};

use gl::UseProgram;

use crate::{Camera, Light, LightHandle, Mesh, MeshHandle, DEFAULT_SHADER};

pub struct Renderer {
    pub meshes: HashMap<MeshHandle, Mesh>,
    pub lights: HashMap<LightHandle, Light>,
    pub camera: Camera,
} 

impl Renderer {
    pub fn new() -> Self {
        let camera = Camera::new();
        Self {
            camera,
            meshes: HashMap::new(),
            lights: HashMap::new(),
        }
    }

    pub unsafe fn draw(&mut self) {
        DEFAULT_SHADER.use_shader();
        self.camera.send_uniforms(&DEFAULT_SHADER);
        self.send_light_uniforms(&DEFAULT_SHADER);
        UseProgram(0);

        for mesh in self.meshes.values_mut() {
            if mesh.has_been_set_up == false {
                mesh.setup_mesh();
            }
            mesh.draw();
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