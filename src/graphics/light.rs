use gl::UseProgram;
use glam::Vec3;

use std::{collections::HashMap, ffi::CString, ops::{Index, IndexMut}};

use crate::{cstr, Renderer, Shader};

#[derive(Copy, Clone)]
pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
}
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct LightHandle {
    pub id: usize,
}

impl Index<LightHandle> for HashMap<LightHandle, Light> {
    type Output = Light;

    fn index(&self, handle: LightHandle) -> &Self::Output {
        self.get(&handle).expect("No entry found for key")
    }
}

impl IndexMut<LightHandle> for HashMap<LightHandle, Light> {
    fn index_mut(&mut self, handle: LightHandle) -> &mut Self::Output {
        self.get_mut(&handle).expect("No entry found for key")
    }
}

impl Renderer {
    pub unsafe fn send_light_uniforms(&self, shader: &Shader) {
        shader.use_shader();
        shader.uniform_vec3f(cstr!("viewPos"), &self.camera.pos);
        shader.uniform_1i(cstr!("num_lights"), self.lights.len() as i32);
        let mut i = 0;
        for light in self.lights.values() {
            shader.uniform_vec3f(cstr!(format!("lightColor[{}]", i)), &light.color);
            shader.uniform_vec3f(cstr!(format!("lightPos[{}]", i)), &light.position);

            i+=1;
        }
        UseProgram(0);
    }

    pub fn add_light(&mut self, light: Light) -> Option<LightHandle> {
        let handle = LightHandle {id: self.lights.len()};

        if self.lights.contains_key(&handle) {
            println!("Light with handle {:?} already exists", handle);
            return None;
        }

        self.lights.insert(handle, light);
        Some(handle)
    }

    pub fn destroy_light(&mut self, handle: LightHandle) {
        if self.lights.remove(&handle).is_some() {

        } else {
            println!("Failed to remove light");
        }
    }
}