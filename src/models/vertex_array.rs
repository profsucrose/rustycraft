use gl::types::*;
pub struct VertexArray {
    vao: GLuint
}

impl VertexArray {
    pub unsafe fn new() -> VertexArray {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);       
        VertexArray { vao }
    }

    pub unsafe fn bind(&self) {
        gl::BindVertexArray(self.vao);
    }

    pub unsafe fn unbind() {
        gl::BindVertexArray(0);
    }
}