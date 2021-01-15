use std::{ffi::c_void, ptr};

use gl::types::*;

use super::{shader::Shader, vertex_array::VertexArray, vertex_buffer::VertexBuffer};

pub struct FrameBuffer {
    id: GLuint,
    vao: VertexArray,
    vbo: VertexBuffer,
    shader: Shader,
    tex_color_buffer: GLuint
}

impl FrameBuffer {
    // creates and binds framebuffer object
    pub unsafe fn new(width: u32, height: u32) -> FrameBuffer {
        let mut id = 0;
        gl::GenFramebuffers(1, &mut id);

        // bind for attachment
        gl::BindFramebuffer(gl::FRAMEBUFFER, id);

        // texture color attachment
        let mut tex_color_buffer = 0;
        gl::GenTextures(1, &mut tex_color_buffer);
        gl::BindTexture(gl::TEXTURE_2D, tex_color_buffer);
        
        // allocates empty texture buffer so 
        // make GL calls directly instead of using 
        // abstracted class
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, width as GLint, height as GLint, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null());

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);  
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex_color_buffer, 0);

        // render buffer object for depth and stencil buffers
        let mut rbo = 0;
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER,  gl::DEPTH24_STENCIL8, width as GLint, height as GLint);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);

        if FrameBuffer::is_complete() {
            println!("Framebuffer is complete!");
        }

        FrameBuffer::unbind();

        let vao = VertexArray::new();
        vao.bind();

        let mut vbo = VertexBuffer::new();
        vbo.bind();

        vbo.add_float_attribute(2, 4);
        vbo.add_float_attribute(2, 4);

        let vertices: Vec<f32> = vec![
            // positions // uvs
            -1.0,  1.0,  0.0, 1.0,
            -1.0, -1.0,  0.0, 0.0,
             1.0, -1.0,  1.0, 0.0,
        
            -1.0,  1.0,  0.0, 1.0,
             1.0, -1.0,  1.0, 0.0,
             1.0,  1.0,  1.0, 1.0
        ];	

        vbo.set_data(&vertices, gl::STATIC_DRAW);

        let shader = Shader::new("assets/shaders/framebuffer/fb_vertex.vert", "assets/shaders/framebuffer/fb_fragment.frag");

        VertexArray::unbind();
        VertexBuffer::unbind();

        FrameBuffer { id, vao, vbo, shader, tex_color_buffer }
    }

    pub unsafe fn draw(&self) {
        // bind to 0th framebuffer to draw
        FrameBuffer::unbind();
        gl::Disable(gl::DEPTH_TEST);
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        self.shader.use_program();
        self.vao.bind();
        self.vbo.bind();
        gl::BindTexture(gl::TEXTURE_2D, self.tex_color_buffer);
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
    }

    pub unsafe fn bind(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
    }

    pub unsafe fn unbind() {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    pub unsafe fn delete(&self) {
        gl::DeleteFramebuffers(1, &self.id);
    }

    // framebuffer must be bound before checking completeness
    pub unsafe fn is_complete() -> bool {
        gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE
    }
}