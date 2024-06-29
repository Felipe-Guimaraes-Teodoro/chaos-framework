// https://github.com/michaelfairley/rust-imgui-opengl-renderer/blob/master/src/lib.rs

use imgui::Context;
use std::mem;

use gl::*;
use gl::types::*;

pub struct ImguiRenderer {
  program: GLuint,
  locs: Locs,
  vbo: GLuint,
  ebo: GLuint,
  font_texture: GLuint,
}

struct Locs {
  texture: GLint,
  proj_mtx: GLint,
  position: GLuint,
  uv: GLuint,
  color: GLuint,
}

impl ImguiRenderer {
  pub fn new<F>(
    imgui: &mut Context,
    load_fn: F,
  ) -> Self
    where
    F: FnMut(&'static str) -> *const ::std::os::raw::c_void
  {
    let _gl = gl::load_with(load_fn);

    unsafe {
      #[cfg(target_os = "macos")]
      let glsl_version = b"#version 150\n\0";
      #[cfg(not(target_os = "macos"))]
      let glsl_version = b"#version 130\n\0";

      let vert_source = b"
        uniform mat4 ProjMtx;
        in vec2 Position;
        in vec2 UV;
        in vec4 Color;
        out vec2 Frag_UV;
        out vec4 Frag_Color;
        void main()
        {
          Frag_UV = UV;
          Frag_Color = Color;
          gl_Position = ProjMtx * vec4(Position.xy,0,1);
        }
      \0";

      let frag_source = b"
        uniform sampler2D Texture;
        in vec2 Frag_UV;
        in vec4 Frag_Color;
        out vec4 Out_Color;
        void main()
        {
          Out_Color = Frag_Color * texture(Texture, Frag_UV.st);
        }
      \0";

      let vert_sources = [glsl_version.as_ptr() as *const GLchar,
                          vert_source.as_ptr() as *const GLchar];
      let vert_sources_len = [glsl_version.len() as GLint - 1,
                              vert_source.len() as GLint - 1];
      let frag_sources = [glsl_version.as_ptr() as *const GLchar,
                          frag_source.as_ptr() as *const GLchar];
      let frag_sources_len = [glsl_version.len() as GLint - 1,
                              frag_source.len() as GLint - 1];

      let program = CreateProgram();
      let vert_shader = CreateShader(gl::VERTEX_SHADER);
      let frag_shader = CreateShader(gl::FRAGMENT_SHADER);
      ShaderSource(vert_shader, 2, vert_sources.as_ptr(), vert_sources_len.as_ptr());
      ShaderSource(frag_shader, 2, frag_sources.as_ptr(), frag_sources_len.as_ptr());
      CompileShader(vert_shader);
      CompileShader(frag_shader);
      AttachShader(program, vert_shader);
      AttachShader(program, frag_shader);
      LinkProgram(program);
      DeleteShader(vert_shader);
      DeleteShader(frag_shader);

      let locs = Locs{
        texture: GetUniformLocation(program, b"Texture\0".as_ptr() as _),
        proj_mtx: GetUniformLocation(program, b"ProjMtx\0".as_ptr() as _),
        position: GetAttribLocation(program, b"Position\0".as_ptr() as _) as _,
        uv: GetAttribLocation(program, b"UV\0".as_ptr() as _) as _,
        color: GetAttribLocation(program, b"Color\0".as_ptr() as _) as _,
      };

      let vbo = return_param(|x| GenBuffers(1, x) );
      let ebo = return_param(|x| GenBuffers(1, x) );

      let mut current_texture = 0;
      GetIntegerv(gl::TEXTURE_BINDING_2D, &mut current_texture);


      let font_texture = return_param(|x| GenTextures(1, x));
      BindTexture(gl::TEXTURE_2D, font_texture);
      TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
      TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
      PixelStorei(gl::UNPACK_ROW_LENGTH, 0);

      {
        let atlas = imgui.fonts();

        let texture = atlas.build_rgba32_texture();
        TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _, texture.width as _, texture.height as _, 0, gl::RGBA, gl::UNSIGNED_BYTE, texture.data.as_ptr() as _);

        atlas.tex_id = (font_texture as usize).into();
      }

      BindTexture(gl::TEXTURE_2D, current_texture as _);


      Self{
        program,
        locs,
        vbo,
        ebo,
        font_texture,
      }
    }
  }

  pub fn render(
    &self,
    ctx: &mut Context,
  ) {
    use imgui::{DrawVert,DrawIdx,DrawCmd,DrawCmdParams};

    unsafe {
      let last_active_texture = return_param(|x| GetIntegerv(gl::ACTIVE_TEXTURE, x));
      ActiveTexture(gl::TEXTURE0);
      let last_program = return_param(|x| GetIntegerv(gl::CURRENT_PROGRAM, x));
      let last_texture = return_param(|x| GetIntegerv(gl::TEXTURE_BINDING_2D, x));
      let last_sampler = if BindSampler::is_loaded() { return_param(|x| GetIntegerv(gl::SAMPLER_BINDING, x)) } else { 0 };
      let last_array_buffer = return_param(|x| GetIntegerv(gl::ARRAY_BUFFER_BINDING, x));
      let last_element_array_buffer = return_param(|x| GetIntegerv(gl::ELEMENT_ARRAY_BUFFER_BINDING, x));
      let last_vertex_array = return_param(|x| GetIntegerv(gl::VERTEX_ARRAY_BINDING, x));
      let last_polygon_mode = return_param(|x: &mut [GLint; 2]| GetIntegerv(gl::POLYGON_MODE, x.as_mut_ptr()));
      let last_viewport = return_param(|x: &mut [GLint; 4]| GetIntegerv(gl::VIEWPORT, x.as_mut_ptr()));
      let last_scissor_box = return_param(|x: &mut [GLint; 4]| GetIntegerv(gl::SCISSOR_BOX, x.as_mut_ptr()));
      let last_blend_src_rgb = return_param(|x| GetIntegerv(gl::BLEND_SRC_RGB, x));
      let last_blend_dst_rgb = return_param(|x| GetIntegerv(gl::BLEND_DST_RGB, x));
      let last_blend_src_alpha = return_param(|x| GetIntegerv(gl::BLEND_SRC_ALPHA, x));
      let last_blend_dst_alpha = return_param(|x| GetIntegerv(gl::BLEND_DST_ALPHA, x));
      let last_blend_equation_rgb = return_param(|x| GetIntegerv(gl::BLEND_EQUATION_RGB, x));
      let last_blend_equation_alpha = return_param(|x| GetIntegerv(gl::BLEND_EQUATION_ALPHA, x));
      let last_enable_blend = IsEnabled(gl::BLEND) == gl::TRUE;
      let last_enable_cull_face = IsEnabled(gl::CULL_FACE) == gl::TRUE;
      let last_enable_depth_test = IsEnabled(gl::DEPTH_TEST) == gl::TRUE;
      let last_enable_scissor_test = IsEnabled(gl::SCISSOR_TEST) == gl::TRUE;

      Enable(gl::BLEND);
      BlendEquation(gl::FUNC_ADD);
      BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
      Disable(gl::CULL_FACE);
      Disable(gl::DEPTH_TEST);
      Enable(gl::SCISSOR_TEST);
      PolygonMode(gl::FRONT_AND_BACK, gl::FILL);

      let [width, height] = ctx.io().display_size;
      let [scale_w, scale_h] = ctx.io().display_framebuffer_scale;

      let fb_width = width * scale_w;
      let fb_height = height * scale_h;

      Viewport(0, 0, fb_width as _, fb_height as _);
      let matrix = [
        [ 2.0 / width as f32, 0.0,                     0.0, 0.0],
        [ 0.0,                2.0 / -(height as f32),  0.0, 0.0],
        [ 0.0,                0.0,                    -1.0, 0.0],
        [-1.0,                1.0,                     0.0, 1.0],
      ];
      UseProgram(self.program);
      Uniform1i(self.locs.texture, 0);
      UniformMatrix4fv(self.locs.proj_mtx, 1, gl::FALSE, matrix.as_ptr() as _);
      if BindSampler::is_loaded() { BindSampler(0, 0); }


      let vao = return_param(|x| GenVertexArrays(1, x));
      BindVertexArray(vao);
      BindBuffer(gl::ARRAY_BUFFER, self.vbo);
      EnableVertexAttribArray(self.locs.position);
      EnableVertexAttribArray(self.locs.uv);
      EnableVertexAttribArray(self.locs.color);
      VertexAttribPointer(self.locs.position, 2, gl::FLOAT,         gl::FALSE, mem::size_of::<DrawVert>() as _, field_offset::<DrawVert, _, _>(|v| &v.pos) as _);
      VertexAttribPointer(self.locs.uv,       2, gl::FLOAT,         gl::FALSE, mem::size_of::<DrawVert>() as _, field_offset::<DrawVert, _, _>(|v| &v.uv) as _);
      VertexAttribPointer(self.locs.color,    4, gl::UNSIGNED_BYTE, gl::TRUE,  mem::size_of::<DrawVert>() as _, field_offset::<DrawVert, _, _>(|v| &v.col) as _);


      let draw_data = ctx.render();

      for draw_list in draw_data.draw_lists() {
        let vtx_buffer = draw_list.vtx_buffer();
        let idx_buffer = draw_list.idx_buffer();

        BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        BufferData(gl::ARRAY_BUFFER, (vtx_buffer.len() * mem::size_of::<DrawVert>()) as _, vtx_buffer.as_ptr() as _, gl::STREAM_DRAW);

        BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
        BufferData(gl::ELEMENT_ARRAY_BUFFER, (idx_buffer.len() * mem::size_of::<DrawIdx>()) as _, idx_buffer.as_ptr() as _, gl::STREAM_DRAW);

        for cmd in draw_list.commands() {
          match cmd {
            DrawCmd::Elements {
              count,
              cmd_params: DrawCmdParams {
                clip_rect: [x, y, z, w],
                texture_id,
                idx_offset,
                ..
              },
            } => {
              BindTexture(gl::TEXTURE_2D, texture_id.id() as _);

              Scissor((x * scale_w) as GLint,
                         (fb_height - w * scale_h) as GLint,
                         ((z - x) * scale_w) as GLint,
                         ((w - y) * scale_h) as GLint);

              let idx_size = if mem::size_of::<DrawIdx>() == 2 { gl::UNSIGNED_SHORT } else { gl::UNSIGNED_INT };

              DrawElements(gl::TRIANGLES, count as _, idx_size, (idx_offset * mem::size_of::<DrawIdx>()) as _);
            },
            DrawCmd::ResetRenderState => {
              unimplemented!("Haven't implemented DrawCmd::ResetRenderState yet");
            },
            DrawCmd::RawCallback { .. } => {
              unimplemented!("Haven't implemented user callbacks yet");
            }
          }
        }
      }

      DeleteVertexArrays(1, &vao);

      UseProgram(last_program as _);
      BindTexture(gl::TEXTURE_2D, last_texture as _);
      if BindSampler::is_loaded() { BindSampler(0, last_sampler as _); }
      ActiveTexture(last_active_texture as _);
      BindVertexArray(last_vertex_array as _);
      BindBuffer(gl::ARRAY_BUFFER, last_array_buffer as _);
      BindBuffer(gl::ELEMENT_ARRAY_BUFFER, last_element_array_buffer as _);
      BlendEquationSeparate(last_blend_equation_rgb as _, last_blend_equation_alpha as _);
      BlendFuncSeparate(last_blend_src_rgb as _, last_blend_dst_rgb as _, last_blend_src_alpha as _, last_blend_dst_alpha as _);
      if last_enable_blend { Enable(gl::BLEND) } else { Disable(gl::BLEND) };
      if last_enable_cull_face { Enable(gl::CULL_FACE) } else { Disable(gl::CULL_FACE) };
      if last_enable_depth_test { Enable(gl::DEPTH_TEST) } else { Disable(gl::DEPTH_TEST) };
      if last_enable_scissor_test { Enable(gl::SCISSOR_TEST) } else { Disable(gl::SCISSOR_TEST) };
      PolygonMode(gl::FRONT_AND_BACK, last_polygon_mode[0] as _);
      Viewport(last_viewport[0] as _, last_viewport[1] as _, last_viewport[2] as _, last_viewport[3] as _);
      Scissor(last_scissor_box[0] as _, last_scissor_box[1] as _, last_scissor_box[2] as _,  last_scissor_box[3] as _);
    }
  }
}

impl Drop for ImguiRenderer {
  fn drop(&mut self) {
    unsafe {
      DeleteBuffers(1, &self.vbo);
      DeleteBuffers(1, &self.ebo);

      DeleteProgram(self.program);

      DeleteTextures(1, &self.font_texture);
    }
  }
}

fn field_offset<T, U, F: for<'a> FnOnce(&'a T) -> &'a U>(f: F) -> usize {
  unsafe {
    let instance = mem::zeroed::<T>();

    let offset = {
      let field: &U = f(&instance);
      field as *const U as usize - &instance as *const T as usize
    };

    mem::forget(instance);

    offset
  }
}

fn return_param<T, F>(f: F) -> T where F: FnOnce(&mut T) {
  let mut val = unsafe{ mem::zeroed() };
  f(&mut val);
  val
}