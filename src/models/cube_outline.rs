use super::{vertex_array::VertexArray, vertex_buffer::VertexBuffer};

pub struct CubeOutline {
    vao: VertexArray,
    vbo: VertexBuffer
}

impl CubeOutline {
    pub unsafe fn new() -> CubeOutline {
        let vao = VertexArray::new();
        vao.bind();

        let vbo = VertexBuffer::new();
        vbo.bind();
        
        CubeOutline { vao, vbo }
    }
}