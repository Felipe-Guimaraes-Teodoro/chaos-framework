use gl::*;

use crate::cstr;

use super::{renderer, Renderer, Shader, DEFAULT_SHADER};

use std::ffi::CString;

pub struct Framebuffer {
    fbo: u32,
    rbo: u32,

    current_frame: u32,
    prev_frame: u32,
    depth_texture: u32,
    quad_vao: u32,

    shader: Shader,
}

impl Framebuffer {
    pub fn new_texture(w: i32, h: i32) -> Self {
        let mut fbo = 0;
        let mut rbo = 0;
        let mut current_frame = 0;
        let mut prev_frame = 0;
        let mut depth_texture = 0;

        let shader = Shader::new_pipeline(TEST_VS, TEST_FS);
        
        unsafe {
            GenFramebuffers(1, &mut fbo);
            // GenRenderbuffers(1, &mut rbo);
            // BindRenderbuffer(RENDERBUFFER, rbo);
            BindFramebuffer(FRAMEBUFFER, fbo);

            GenTextures(1, &mut current_frame);
            BindTexture(TEXTURE_2D, current_frame);
            TexImage2D(TEXTURE_2D, 0, RGB as i32, w, h, 0, RGB, UNSIGNED_BYTE, std::ptr::null());
            TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as i32);
            BindTexture(TEXTURE_2D, 0);

            GenTextures(1, &mut prev_frame);
            BindTexture(TEXTURE_2D, prev_frame);
            TexImage2D(TEXTURE_2D, 0, RGB8 as i32, w, h, 0, RGB, UNSIGNED_BYTE, std::ptr::null());
            TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as i32);
            BindTexture(TEXTURE_2D, 0);

            GenTextures(1, &mut depth_texture);
            BindTexture(TEXTURE_2D, depth_texture);
            TexImage2D(TEXTURE_2D, 0, DEPTH_COMPONENT as i32, w, h, 0, DEPTH_COMPONENT, FLOAT, std::ptr::null());
            TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as i32);
            BindTexture(TEXTURE_2D, 0);

            shader.use_shader();
            shader.uniform_1i(cstr!("screenBuffer"), 0);
            shader.uniform_1i(cstr!("backBuffer"), 1);
            shader.uniform_1i(cstr!("depthBuffer"), 2);
            UseProgram(0);

            BindFramebuffer(FRAMEBUFFER, fbo);
            FramebufferTexture2D(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, current_frame, 0);
            FramebufferTexture2D(FRAMEBUFFER, DEPTH_ATTACHMENT, TEXTURE_2D, depth_texture, 0);

            //RenderbufferStorage(RENDERBUFFER, DEPTH24_STENCIL8, w, h);
            //FramebufferRenderbuffer(FRAMEBUFFER, DEPTH_STENCIL_ATTACHMENT, RENDERBUFFER, rbo);

            
            // BindRenderbuffer(RENDERBUFFER, 0);
            BindFramebuffer(FRAMEBUFFER, 0);
        }

        Self {
            fbo,
            rbo,
            quad_vao: Self::quad_vao(),
            current_frame,
            prev_frame,
            depth_texture,
            shader,
        }
    }

    pub fn draw_first_pass(&self, renderer: &mut Renderer) {
        unsafe { 
            BindFramebuffer(FRAMEBUFFER, self.fbo);
            Enable(BLEND);
            BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
        };
    }

    pub fn draw_second_pass(&mut self) {
        unsafe {
            BindFramebuffer(FRAMEBUFFER, 0); // bind default framebuffer
            Clear(DEPTH_BUFFER_BIT);
    
            self.shader.use_shader();
            self.shader.uniform_1i(cstr!("screenBuffer"), 0);
            self.shader.uniform_1i(cstr!("backBuffer"), 1);
            self.shader.uniform_1i(cstr!("depthBuffer"), 2);
    
            ActiveTexture(TEXTURE0);
            BindTexture(TEXTURE_2D, self.current_frame);
            ActiveTexture(TEXTURE1);
            BindTexture(TEXTURE_2D, self.prev_frame);
            ActiveTexture(TEXTURE2);
            BindTexture(TEXTURE_2D, self.depth_texture);
            
            BindVertexArray(self.quad_vao); 
            Disable(DEPTH_TEST); 
            DrawArrays(TRIANGLES, 0, 6); 
            UseProgram(0);
        }
    }

    pub fn delete(&self) {
        unsafe {
            DeleteFramebuffers(1, &self.fbo);
            DeleteRenderbuffers(1, &self.rbo);
        }
    }

    fn quad_vao() -> u32 {
        let quad_vertices: [f32; 24] = [
            // pos         // texCoords
             1.0, -1.0,    1.0, 0.0, // bottom right
            -1.0,  1.0,    0.0, 1.0, // top left
             1.0,  1.0,    1.0, 1.0, // top right
    
             1.0, -1.0,    1.0, 0.0, // bottom right
            -1.0, -1.0,    0.0, 0.0, // bottom left
            -1.0,  1.0,    0.0, 1.0, // top left
        ];
    
        let mut quad_vao: u32 = 0;
        let mut quad_vbo: u32 = 0;
    
        unsafe {
            GenVertexArrays(1, &mut quad_vao);
            GenBuffers(1, &mut quad_vbo);
    
            BindVertexArray(quad_vao);
    
            BindBuffer(ARRAY_BUFFER, quad_vbo);
            BufferData(
                ARRAY_BUFFER,
                (quad_vertices.len() * std::mem::size_of::<f32>()) as isize,
                quad_vertices.as_ptr() as *const _,
                STATIC_DRAW,
            );
    
            EnableVertexAttribArray(0);
            VertexAttribPointer(
                0, 
                2, 
                FLOAT, 
                FALSE, 
                4 * std::mem::size_of::<f32>() as i32, 
                std::ptr::null()
            );
    
            EnableVertexAttribArray(1);
            VertexAttribPointer(
                1, 
                2, 
                FLOAT, 
                FALSE, 
                4 * std::mem::size_of::<f32>() as i32, 
                (2 * std::mem::size_of::<f32>()) as isize as *const std::ffi::c_void
            );

            BindBuffer(ARRAY_BUFFER, 0); 
            BindVertexArray(0); 
        }
    
        quad_vao
    }
}

pub const TEST_VS: &str = r#"
#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoords;

out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;

void main()
{
    gl_Position = vec4(aPos, 0.0, 1.0);
    TexCoords = aTexCoords;
}
"#;


pub const TEST_FS: &str = r#"
#version 330 core
out vec4 FragColor;
  
in vec2 TexCoords;

uniform sampler2D screenBuffer;
uniform sampler2D backBuffer;
uniform sampler2D depthBuffer;

void main()
{
    FragColor = vec4(vec3(texture(screenBuffer, TexCoords)), 0.5);
}
"#;
