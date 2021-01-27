#![allow(dead_code)]
use std::ptr;
use gl::types::*;

pub const SHADOW_WIDTH: GLint = 10024;
pub const SHADOW_HEIGHT: GLint = 10024;

// framebuffer for getting shadow map 
// for shadow calculations
pub struct DepthFrameBuffer {
    id: GLuint,
    pub depth_map: GLuint
}

impl DepthFrameBuffer {
    // creates and binds framebuffer object
    pub unsafe fn new() -> DepthFrameBuffer {
        let mut id = 0;
        gl::GenFramebuffers(1, &mut id);

        // bind for attachment
        gl::BindFramebuffer(gl::FRAMEBUFFER, id);

        // depth map attachment
        let mut depth_map = 0;
        gl::GenTextures(1, &mut depth_map);
        gl::BindTexture(gl::TEXTURE_2D, depth_map);

        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as GLint, SHADOW_WIDTH, SHADOW_HEIGHT, 0, gl::DEPTH_COMPONENT, gl::FLOAT, ptr::null());

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);  
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);  
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);  

        let border_color = [ 1.0, 1.0, 1.0, 1.0 ];
        gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, border_color.as_ptr());

        // attach depth map
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_map, 0);

        // tell OpenGL we're not going to attach any color buffer
        // as we only need depth
        gl::DrawBuffer(gl::NONE);
        gl::DrawBuffer(gl::NONE);

        if DepthFrameBuffer::is_complete() {
            println!("Depth framebuffer is complete!");
        }

        DepthFrameBuffer::unbind();

        DepthFrameBuffer { id, depth_map }
    }

    pub unsafe fn bind(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
    }

    pub unsafe fn unbind() {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    pub unsafe fn delete(&self) {
        gl::DeleteFramebuffers(1, &self.id);
    }

    // framebuffer must be bound before checking completeness
    pub unsafe fn is_complete() -> bool {
        gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE
    }
}