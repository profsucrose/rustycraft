use gl::types::*;
use std::ffi::c_void;

pub struct VertexBuffer {
    vbo: GLuint,
    attribute_index: u32,
    attribute_offset: usize
}

impl VertexBuffer {
    pub unsafe fn new() -> VertexBuffer {
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);  
        VertexBuffer { vbo, attribute_index: 0, attribute_offset: 0 }
    }

    pub unsafe fn unbind() {
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    pub unsafe fn bind(&self) {
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
    }

    pub unsafe fn add_float_attribute(&mut self, length: usize, stride: usize) {
        gl::VertexAttribPointer(
            self.attribute_index, 
            length as GLint, 
            gl::FLOAT, 
            gl::FALSE, 
            (stride * std::mem::size_of::<GLfloat>()) as GLsizei, 
            if self.attribute_offset == 0 {
                std::ptr::null()
            } else {
                (self.attribute_offset * std::mem::size_of::<GLfloat>()) as *const c_void
            }
        );
        gl::EnableVertexAttribArray(self.attribute_index);
        self.attribute_index += 1;
        self.attribute_offset += length;
    }

    pub unsafe fn set_data(&self, vertices: &Vec<f32>, flag: GLuint) {
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            vertices.as_ptr() as *const c_void, 
            flag
        );
    }
}