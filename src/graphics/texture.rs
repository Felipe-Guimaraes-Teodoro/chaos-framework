use std::{collections::HashMap, ops::{Index, IndexMut}};

use gl::types::{GLint, GLsizei, GLuint, GLvoid};

use crate::Renderer;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct TextureHandle {
    pub id: usize,
}

impl<'a> Index<TextureHandle> for HashMap<TextureHandle, Texture<'a>> {
    type Output = Texture<'a>;

    fn index(&self, handle: TextureHandle) -> &Self::Output {
        self.get(&handle).expect("No entry found for key")
    }
}

impl<'a> IndexMut<TextureHandle> for HashMap<TextureHandle, Texture<'a>> {
    fn index_mut(&mut self, handle: TextureHandle) -> &mut Self::Output {
        self.get_mut(&handle).expect("No entry found for key")
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Texture<'a> {
    Path(&'a str),
    Loaded(GLuint),
    None,
}

pub unsafe fn load_texture(path: &str) -> GLuint { 
    let img = image::open(path).expect("Failed to load image");
    
    let img = img.flipv();
    let width = img.width();
    let height = img.height();
    let raw_pixels = img.to_rgba8().into_raw();
    
    let mut texture: GLuint = 0;
    gl::GenTextures(1, &mut texture);
    gl::BindTexture(gl::TEXTURE_2D, texture);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA as GLint,
        width as GLsizei,
        height as GLsizei,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        raw_pixels.as_ptr() as *const GLvoid,
    );

    gl::GenerateMipmap(gl::TEXTURE_2D);

    texture
}

impl Renderer {
    pub fn add_texture(&mut self, path: &str) -> Option<TextureHandle> {
        let handle = TextureHandle {id: self.textures.len()};

        if self.textures.contains_key(&handle) {
            println!("Texture with handle {:?} already exists", handle);
            return None;
        }

        self.textures.insert(handle, unsafe {load_texture(path)});
        Some(handle)
    }

    pub fn destroy_texture(&mut self, handle: TextureHandle) {
        if self.textures.remove(&handle).is_some() {

        } else {
            println!("Failed to remove texture");
        }
    }
}

impl Drop for Texture<'_> {
    fn drop(&mut self) {
        unsafe {
            if let Texture::Loaded(id) = self {
                gl::DeleteTextures(1, id);
            }
        }
    }
}