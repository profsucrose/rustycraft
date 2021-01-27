#![allow(dead_code)]
use cgmath::{Matrix4, Vector3};
use crate::opengl::{camera::Camera, shader::Shader, vertex_array::VertexArray, vertex_buffer::VertexBuffer};

pub struct Cloud { 
    vao: VertexArray,
    vbo: VertexBuffer,
    shader: Shader
}

impl Cloud {
    pub unsafe fn new() -> Cloud {
        let vao = VertexArray::new();
        vao.bind();

        let mut vbo = VertexBuffer::new();
        let vertices = vec!(
            // front
            -0.5, -0.5, -0.5,
             0.5, -0.5, -0.5,
             0.5,  0.5, -0.5,
             0.5,  0.5, -0.5,
            -0.5,  0.5, -0.5,
            -0.5, -0.5, -0.5,

            // back
            -0.5, -0.5,  0.5,  
            0.5, -0.5,  0.5,   
            0.5,  0.5,  0.5,   
            0.5,  0.5,  0.5,   
            -0.5,  0.5,  0.5,  
            -0.5, -0.5,  0.5,  

            // left
            -0.5,  0.5,  0.5,  
            -0.5,  0.5, -0.5,  
            -0.5, -0.5, -0.5,  
            -0.5, -0.5, -0.5,  
            -0.5, -0.5,  0.5,  
            -0.5,  0.5,  0.5,  

            // right
            0.5,  0.5,  0.5,  
            0.5,  0.5, -0.5,  
            0.5, -0.5, -0.5,  
            0.5, -0.5, -0.5,  
            0.5, -0.5,  0.5,  
            0.5,  0.5,  0.5,  

            // bottom
            -0.5, -0.5, -0.5,  
            0.5, -0.5, -0.5,  
            0.5, -0.5,  0.5,  
            0.5, -0.5,  0.5,  
            -0.5, -0.5,  0.5,  
            -0.5, -0.5, -0.5,  

            // top
            -0.5,  0.5, -0.5,  
            0.5,  0.5, -0.5,   
            0.5,  0.5,  0.5,   
            0.5,  0.5,  0.5,   
            -0.5,  0.5,  0.5,  
            -0.5,  0.5, -0.5,  
        );
        vbo.bind();
        vbo.set_data(&vertices, gl::STATIC_DRAW);
        
        // position
        vbo.add_float_attribute(3, 3);

        VertexArray::unbind();
        VertexBuffer::unbind();

        let shader = Shader::new("assets/shaders/cloud/cloud_vertex.vert", "assets/shaders/cloud/cloud_fragment.frag");
        Cloud { vao, vbo, shader }
    }

    pub unsafe fn draw(&self, camera: &Camera, x: i32, y: i32, z: i32, z_offset: f32) {
        self.shader.use_program();
        self.shader.set_mat4("view", camera.get_view());
        self.shader.set_mat4("model", Matrix4::from_translation(Vector3::new(x as f32 * 10.0, y as f32, z as f32 * 10.0 + z_offset)) * Matrix4::from_nonuniform_scale(10.0, 2.0, 10.0));
        self.shader.set_mat4("projection", camera.get_projection());

        self.vao.bind();
        self.vbo.bind();
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
        
        VertexArray::unbind();
        VertexBuffer::unbind();
    }
}