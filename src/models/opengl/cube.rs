#![allow(dead_code)]
use std::ptr;

use cgmath::{Matrix4, Vector3};

use crate::models::opengl::camera::Camera;

use super::{element_buffer::ElementBuffer, face_uvs::FaceUVs, shader::Shader, texture::Texture, vertex_array::VertexArray, vertex_buffer::VertexBuffer};

pub struct Cube {
    vao: VertexArray,
    vbo: VertexBuffer,
    shader: Shader,
    texture: Texture
}

impl Cube {
    pub unsafe fn new(
        texture: Texture, 
        face_front: FaceUVs,
        face_back: FaceUVs,
        face_left: FaceUVs,
        face_right: FaceUVs,
        face_bottom: FaceUVs,
        face_top: FaceUVs
    ) -> Cube {
        let vao = VertexArray::new();
        vao.bind();

        let mut vbo = VertexBuffer::new();
        let vertices = vec!(
            // front
            -0.5, -0.5, -0.5,  face_front.left,  face_front.bottom,
             0.5, -0.5, -0.5,  face_front.right, face_front.bottom,
             0.5,  0.5, -0.5,  face_front.right, face_front.top,
             0.5,  0.5, -0.5,  face_front.right, face_front.top,
            -0.5,  0.5, -0.5,  face_front.left,  face_front.top,
            -0.5, -0.5, -0.5,  face_front.left,  face_front.bottom,

            // back
            -0.5, -0.5,  0.5,  face_back.left, face_back.bottom,
            0.5, -0.5,  0.5,   face_back.right, face_back.bottom,
            0.5,  0.5,  0.5,   face_back.right, face_back.top,
            0.5,  0.5,  0.5,   face_back.right, face_back.top,
            -0.5,  0.5,  0.5,  face_back.left, face_back.top,
            -0.5, -0.5,  0.5,  face_back.left, face_back.bottom,

            // left
            -0.5,  0.5,  0.5,  face_left.left, face_back.top,
            -0.5,  0.5, -0.5,  face_left.right, face_back.top,
            -0.5, -0.5, -0.5,  face_left.right, face_back.bottom,
            -0.5, -0.5, -0.5,  face_left.right, face_back.bottom,
            -0.5, -0.5,  0.5,  face_left.left, face_back.bottom,
            -0.5,  0.5,  0.5,  face_left.left, face_back.top,

            // right
            0.5,  0.5,  0.5,  face_right.left, face_right.top,
            0.5,  0.5, -0.5,  face_right.right, face_right.top,
            0.5, -0.5, -0.5,  face_right.right, face_right.bottom,
            0.5, -0.5, -0.5,  face_right.right, face_right.bottom,
            0.5, -0.5,  0.5,  face_right.left, face_right.bottom,
            0.5,  0.5,  0.5,  face_right.left, face_right.top,

            // bottom
            -0.5, -0.5, -0.5,  face_bottom.left, face_bottom.bottom,
            0.5, -0.5, -0.5,  face_bottom.right, face_bottom.bottom,
            0.5, -0.5,  0.5,  face_bottom.right, face_bottom.top,
            0.5, -0.5,  0.5,  face_bottom.right, face_bottom.top,
            -0.5, -0.5,  0.5,  face_bottom.left, face_bottom.top,
            -0.5, -0.5, -0.5,  face_bottom.left, face_bottom.bottom,

            // top
            -0.5,  0.5, -0.5,  face_top.left, face_top.bottom,
            0.5,  0.5, -0.5,   face_top.right, face_top.bottom,
            0.5,  0.5,  0.5,   face_top.right, face_top.top,
            0.5,  0.5,  0.5,   face_top.right, face_top.top,
            -0.5,  0.5,  0.5,  face_top.left, face_top.top,
            -0.5,  0.5, -0.5,  face_top.left, face_top.bottom
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
        Cube { vao, vbo, shader, texture }
    }

    pub unsafe fn draw(&self, camera: &Camera, model: Matrix4<f32>) {
        self.texture.bind();

        self.shader.use_program();
        self.shader.set_mat4("model", model);
        self.shader.set_mat4("view", camera.get_view());
        self.shader.set_mat4("projection", camera.get_projection());
        self.shader.set_texture("player_texture", &self.texture);

        self.vao.bind();
        self.vbo.bind();
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
        
        VertexArray::unbind();
        VertexBuffer::unbind();
        ElementBuffer::unbind();
    }
}