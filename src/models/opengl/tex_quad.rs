use cgmath::Matrix4;
use gl::types::*;
use std::rc::Rc;

use super::{shader::Shader, texture::Texture, vertex_array::VertexArray, vertex_buffer::VertexBuffer};

pub struct TexQuad {
    texture: Texture,
    vao: VertexArray,
    vbo: VertexBuffer,
    shader: Shader,
    screen_width: u32,
    screen_height: u32
}

impl TexQuad {
    pub unsafe fn new(texture_path: &str, texture_id: GLuint, flipped: bool, screen_width: u32, screen_height: u32) -> TexQuad {
        let texture = Texture::new(texture_path, texture_id, flipped);
        let shader = Shader::new("assets/shaders/texquad/texquad_vertex.vert", "assets/shaders/texquad/texquad_fragment.frag");
        let vao = VertexArray::new();
        let mut vbo = VertexBuffer::new();
        vao.bind();
        vbo.bind();
        vbo.add_float_attribute(2, 4);
        vbo.add_float_attribute(2, 4);
        VertexArray::unbind();
        VertexBuffer::unbind();

        TexQuad { texture, vao, vbo, shader, screen_width, screen_height }
    }

    pub unsafe fn draw(&self, left_x: f32, bottom_y: f32, right_x: f32, top_y: f32, alpha: f32) {
        self.vao.bind();
        let vertices: Vec<f32> = vec![
            // positions        // uvs
            left_x,  top_y,     0.0, 1.0, // top-left
            left_x,  bottom_y,  0.0, 0.0, // bottom-left
            right_x, bottom_y,  1.0, 0.0, // bottom-right
        
            left_x,  top_y,     0.0, 1.0, // top-left
            right_x, bottom_y,  1.0, 0.0, // bottom-right
            right_x, top_y,     1.0, 1.0  // top-right 
        ];	
        self.vbo.bind();
        self.vbo.set_data(&vertices, gl::DYNAMIC_DRAW);

        self.texture.bind();
        self.shader.use_program();
        self.shader.set_mat4("projection", cgmath::ortho(0.0, self.screen_width as f32, 0.0, self.screen_height as f32, -1.0, 100.0));
        self.shader.set_texture("texture", &self.texture);
        self.shader.set_float("alpha", alpha);
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
    }
}