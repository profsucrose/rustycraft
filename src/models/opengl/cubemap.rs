use std::{ffi::c_void, path::Path};

use gl::types::*;
use image::{ColorType, DynamicImage, GenericImage};

#[derive(Clone, Copy)]
pub struct CubeMap {
    id: GLuint 
}

impl CubeMap {
    pub unsafe fn new(image_paths: Vec<&str>, flipped: bool) -> CubeMap {
        let mut id = 0;
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, id);

        for (i, path) in image_paths.iter().enumerate() {
            let mut img = image::open(&Path::new(*path))
            .expect("Failed to load texture");
            if flipped {
                img = img.flipv();
            }
            let data = img.raw_pixels();

            let channels = match img.color() {
                ColorType::RGBA(_) => gl::RGBA,
                ColorType::RGB(_) => gl::RGB,
                _ => panic!(format!("Invalid image color type: {:?}", img.color()))
            };

            // bind image data
            gl::TexImage2D(
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as GLenum, 
                0, 
                channels as i32, 
                img.width() as i32,
                img.height() as i32,
                0,
                channels,
                gl::UNSIGNED_BYTE,
                &data[0] as *const u8 as *const c_void
            );
        }


        // set texture wrapping and filtering options
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as GLint);
        
        CubeMap {
            id
        }
    }

    pub unsafe fn bind(&self) {
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.id);
    }
}