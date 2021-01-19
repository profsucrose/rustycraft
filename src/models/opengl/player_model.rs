use std::ptr;

use cgmath::{Matrix4, Vector3};

use crate::models::opengl::camera::Camera;

use super::{element_buffer::ElementBuffer, shader::Shader, vertex_array::VertexArray, vertex_buffer::VertexBuffer};

pub struct PlayerModel {
    vao: VertexArray,
    vbo: VertexBuffer,
    shader: Shader
}

impl PlayerModel {
    pub unsafe fn new() -> PlayerModel {
        let vao = VertexArray::new();
        vao.bind();

        let mut vbo = VertexBuffer::new();
        let vertices = vec!(
            -0.5, -0.5, -0.5,  0.0, 0.0,
            0.5, -0.5, -0.5,  1.0, 0.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            -0.5,  0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 0.0,

            -0.5, -0.5,  0.5,  0.0, 0.0,
            0.5, -0.5,  0.5,  1.0, 0.0,
            0.5,  0.5,  0.5,  1.0, 1.0,
            0.5,  0.5,  0.5,  1.0, 1.0,
            -0.5,  0.5,  0.5,  0.0, 1.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,

            -0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5,  0.5, -0.5,  1.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,
            -0.5,  0.5,  0.5,  1.0, 0.0,

            0.5,  0.5,  0.5,  1.0, 0.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            0.5, -0.5, -0.5,  0.0, 1.0,
            0.5, -0.5, -0.5,  0.0, 1.0,
            0.5, -0.5,  0.5,  0.0, 0.0,
            0.5,  0.5,  0.5,  1.0, 0.0,

            -0.5, -0.5, -0.5,  0.0, 1.0,
            0.5, -0.5, -0.5,  1.0, 1.0,
            0.5, -0.5,  0.5,  1.0, 0.0,
            0.5, -0.5,  0.5,  1.0, 0.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,

            -0.5,  0.5, -0.5,  0.0, 1.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            0.5,  0.5,  0.5,  1.0, 0.0,
            0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5,  0.5,  0.5,  0.0, 0.0,
            -0.5,  0.5, -0.5,  0.0, 1.0
        );
        vbo.bind();
        vbo.set_data(&vertices, gl::STATIC_DRAW);
        
        // position
        vbo.add_float_attribute(3, 5);
        
        // texcoords
        vbo.add_float_attribute(2, 5);

        VertexArray::unbind();
        VertexBuffer::unbind();

        let shader = Shader::new("assets/shaders/player/player_vertex.vert", "assets/shaders/player/player_fragment.frag");
        PlayerModel { vao, vbo, shader }
    }

    pub unsafe fn draw(&self, camera: &Camera, x: f32, y: f32, z: f32) {
        self.shader.use_program();
        self.shader.set_mat4("model", Matrix4::from_translation(Vector3::new(x, y, z)));
        self.shader.set_mat4("view", camera.get_view());
        self.shader.set_mat4("projection", camera.get_projection());

        self.vao.bind();
        self.vbo.bind();
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
        
        VertexArray::unbind();
        VertexBuffer::unbind();
        ElementBuffer::unbind();
    }
}