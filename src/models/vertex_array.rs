pub struct VertexArray {
    id: u32
}

impl VertexArray {
    pub unsafe fn new() -> VertexArray {
        let mut id = 0;
        gl::GenVertexArrays(1, &mut id);
        VertexArray { id }
    }

    pub unsafe fn bind(&self) {
        gl::BindVertexArray(self.id)
    }
}