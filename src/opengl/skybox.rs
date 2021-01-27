#![allow(dead_code)]
use crate::opengl::{camera::Camera, shader::Shader, vertex_array::VertexArray, vertex_buffer::VertexBuffer};

pub struct SkyBox { 
    vao: VertexArray,
    vbo: VertexBuffer,
    shader: Shader
}

impl SkyBox {
    pub unsafe fn new() -> SkyBox {
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

        let shader = Shader::new("assets/shaders/skybox/skybox_vertex.vert", "assets/shaders/skybox/skybox_fragment.frag");
        SkyBox { vao, vbo, shader }
    }

    pub unsafe fn draw(&self, camera: &Camera) {
        gl::DepthMask(gl::FALSE);
        self.shader.use_program();
        self.shader.set_mat4("view", camera.get_view());
        self.shader.set_mat4("projection", camera.get_projection());

        self.vao.bind();
        self.vbo.bind();
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
        
        VertexArray::unbind();
        VertexBuffer::unbind();
        gl::DepthMask(gl::TRUE);
    }
}