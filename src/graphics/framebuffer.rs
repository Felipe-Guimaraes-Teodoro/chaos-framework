use gl::{BindFramebuffer, BindRenderbuffer, BindTexture, BindVertexArray, Clear, ClearColor, DeleteFramebuffers, DeleteRenderbuffers, Disable, DrawArrays, FramebufferRenderbuffer, FramebufferTexture2D, GenFramebuffers, GenRenderbuffers, GenTextures, RenderbufferStorage, TexImage2D, TexParameteri, COLOR_ATTACHMENT0, COLOR_BUFFER_BIT, DEPTH24_STENCIL8, DEPTH_STENCIL, DEPTH_STENCIL_ATTACHMENT, DEPTH_TEST, FRAMEBUFFER, LINEAR, RENDERBUFFER, RGB, TEXTURE, TEXTURE_2D, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, UNSIGNED_BYTE, UNSIGNED_INT_24_8};

struct Framebuffer {
    fbo: u32,
    rbo: u32,
}

impl Framebuffer {
    pub fn new_depth_texture(w: i32, h: i32) -> Self {
        let mut fbo = 0;
        let mut rbo = 0;
        let mut texture = 0;
        
        unsafe {
            GenFramebuffers(1, &mut fbo);
            GenRenderbuffers(1, &mut rbo);
            BindRenderbuffer(RENDERBUFFER, rbo);
            BindFramebuffer(FRAMEBUFFER, fbo);

            GenTextures(1, &mut texture);
            BindTexture(TEXTURE_2D, texture);
            
            TexImage2D(TEXTURE_2D, 0, RGB as i32, w, h, 0, RGB, UNSIGNED_BYTE, std::ptr::null());
            TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as i32);

            FramebufferTexture2D(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, texture, 0);
            TexImage2D(
                TEXTURE_2D, 0, DEPTH24_STENCIL8 as i32, w, h, 0,
                DEPTH_STENCIL, UNSIGNED_INT_24_8, std::ptr::null(),
            );
            FramebufferTexture2D(FRAMEBUFFER, DEPTH_STENCIL_ATTACHMENT, TEXTURE_2D, texture, 0);

            RenderbufferStorage(RENDERBUFFER, DEPTH24_STENCIL8, w, h);
            FramebufferRenderbuffer(FRAMEBUFFER, DEPTH_STENCIL_ATTACHMENT, RENDERBUFFER, rbo);

            BindFramebuffer(FRAMEBUFFER, 0);
            // DeleteFramebuffers(1, &fbo);
        }

        Self {
            fbo,
            rbo
        }
    }

    pub fn new_texture(w: i32, h: i32) -> Self {
        let mut fbo = 0;
        let mut rbo = 0;
        let mut texture = 0;
        
        unsafe {
            GenFramebuffers(1, &mut fbo);
            GenRenderbuffers(1, &mut rbo);
            BindRenderbuffer(RENDERBUFFER, rbo);
            BindFramebuffer(FRAMEBUFFER, fbo);

            GenTextures(1, &mut texture);
            BindTexture(TEXTURE_2D, texture);
            
            TexImage2D(TEXTURE_2D, 0, RGB as i32, w, h, 0, RGB, UNSIGNED_BYTE, std::ptr::null());
            TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as i32);
            BindTexture(TEXTURE_2D, 0);

            FramebufferTexture2D(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, texture, 0);

            RenderbufferStorage(RENDERBUFFER, DEPTH24_STENCIL8, w, h);
            
            FramebufferRenderbuffer(FRAMEBUFFER, DEPTH_STENCIL_ATTACHMENT, RENDERBUFFER, rbo);

            BindRenderbuffer(RENDERBUFFER, 0);
            BindFramebuffer(FRAMEBUFFER, 0);
            // DeleteFramebuffers(1, &fbo);
        }

        Self {
            fbo,
            rbo
        }
    }

    pub fn draw_first_pass(&self) {
        unsafe { 
            BindFramebuffer(FRAMEBUFFER, self.fbo)

        };
    }

    pub fn draw_second_pass(&self) {
        unsafe {
            BindFramebuffer(FRAMEBUFFER, 0);
            ClearColor(1.0, 1.0, 1.0, 1.0);
            Clear(COLOR_BUFFER_BIT);

            //self.shader.use();
            //BindVertexArray(quad_vao); // screen quad
            Disable(DEPTH_TEST);
            // BindTexture(TEXTURE_2D, self.texture);
            // DrawArrays(TRIANGLES, 0, 6);
        }
    }

    pub fn delete(&self) {
        unsafe {
            DeleteFramebuffers(1, &self.fbo);
            DeleteRenderbuffers(1, &self.rbo);
        }
    }
}

pub const TEST_VS: &str = r#"
#version 330 core
out vec4 FragColor;
  
in vec2 TexCoords;

uniform sampler2D screenTexture;

void main()
{ 
    FragColor = texture(screenTexture, TexCoords);
}
"#;

pub const TEST_FS: &str = r#"

"#;