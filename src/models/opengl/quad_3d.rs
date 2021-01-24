use cgmath::{Matrix4, Vector3};

use crate::models::core::face::Face;

use super::{camera::Camera, shader::Shader, texture::Texture, vertex_array::VertexArray, vertex_buffer::VertexBuffer};

pub struct Quad3d {
    vao: VertexArray,
    vbo: VertexBuffer,
    shader: Shader,
    texture: Texture
}

impl Quad3d {
pub unsafe fn new(face: Face, shader: Shader, texture: Texture, vt_left: f32, vt_bottom: f32, vt_right: f32, vt_top: f32) -> Quad3d {
        let vao = VertexArray::new();
        vao.bind();

        let mut vbo = VertexBuffer::new();
        vbo.bind();
        vbo.add_float_attribute(3, 5);
        vbo.add_float_attribute(2, 5);

        let vertices = match face {
            Face::Top => [
                -0.5,  0.5, -0.5,  vt_left,  vt_bottom, // bottom-left
                 0.5,  0.5, -0.5,  vt_right, vt_bottom, // bottom-right
                 0.5,  0.5,  0.5,  vt_right, vt_top,    // top-right
                 0.5,  0.5,  0.5,  vt_right, vt_top,    // top-right
                -0.5,  0.5,  0.5,  vt_left,  vt_top,    // top-left
                -0.5,  0.5, -0.5,  vt_left,  vt_bottom  // bottom-left
            ],
            Face::Bottom => [
                -0.5, -0.5, -0.5,  vt_left,  vt_bottom, // bottom-left
                 0.5, -0.5, -0.5,  vt_right, vt_bottom, // bottom-right
                 0.5, -0.5,  0.5,  vt_right, vt_top,    // top-right
                 0.5, -0.5,  0.5,  vt_right, vt_top,    // top-right
                -0.5, -0.5,  0.5,  vt_left,  vt_top,    // top-left
                -0.5, -0.5, -0.5,  vt_left,  vt_bottom  // bottom-left
            ],
            Face::Left => [
                -0.5,  0.5,  0.5,  vt_left,  vt_top,    // top-right
                -0.5,  0.5, -0.5,  vt_right, vt_top,    // bottom-right
                -0.5, -0.5, -0.5,  vt_right, vt_bottom, // bottom-left
                -0.5, -0.5, -0.5,  vt_right, vt_bottom, // bottom-left
                -0.5, -0.5,  0.5,  vt_left,  vt_bottom, // bottom-right
                -0.5,  0.5,  0.5,  vt_left,  vt_top     // top-right
            ],
            Face::Right => [
                0.5,  0.5,  0.5,  vt_left,  vt_top,     // top-right
                0.5,  0.5, -0.5,  vt_right, vt_top,     // bottom-right
                0.5, -0.5, -0.5,  vt_right, vt_bottom,  // bottom-left
                0.5, -0.5, -0.5,  vt_right, vt_bottom,  // bottom-left
                0.5, -0.5,  0.5,  vt_left,  vt_bottom,  // bottom-right
                0.5,  0.5,  0.5,  vt_left,  vt_top      // top-right
            ],
            // turn 90 degrees
            Face::Front => [
                -0.5, -0.5, -0.5,  vt_left,  vt_bottom, // bottom-left
                 0.5, -0.5, -0.5,  vt_right, vt_bottom, // bottom-right
                 0.5,  0.5, -0.5,  vt_right, vt_top,    // top-right
                 0.5,  0.5, -0.5,  vt_right, vt_top,    // top-right
                -0.5,  0.5, -0.5,  vt_left,  vt_top,    // top-left
                -0.5, -0.5, -0.5,  vt_left,  vt_bottom  // bottom-left
            ],
            Face::Back => [
                -0.5, -0.5,  0.5,  vt_left,  vt_bottom, // bottom-left
                 0.5, -0.5,  0.5,  vt_right, vt_bottom, // bottom-right
                 0.5,  0.5,  0.5,  vt_right, vt_top,    // top-right
                 0.5,  0.5,  0.5,  vt_right, vt_top,    // top-right
                -0.5,  0.5,  0.5,  vt_left,  vt_top,    // top-left
                -0.5, -0.5,  0.5,  vt_left,  vt_bottom  // bottom-left
            ]
        };
        vbo.set_data(&vertices.to_vec(), gl::STATIC_DRAW);

        VertexArray::unbind();
        VertexBuffer::unbind();
        Quad3d { vao, vbo, shader, texture }
    }

    pub unsafe fn draw(&self, camera: &Camera, x: f32, y: f32, z: f32, scale: Matrix4<f32>) {
        self.vao.bind();
        self.vbo.bind();
        self.texture.bind();
        self.shader.use_program();
        self.shader.set_mat4("view", camera.get_view());
        self.shader.set_mat4("projection", camera.get_projection());
        self.shader.set_mat4("model", Matrix4::from_translation(Vector3::new(x, y, z)));
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
        VertexArray::unbind();
        VertexBuffer::unbind();
    }
}