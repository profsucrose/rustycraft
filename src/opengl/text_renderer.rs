#![allow(dead_code)]
use std::{collections::HashMap, ffi::c_void, ptr};
use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3, ortho, vec2};
use freetype::{Library, ffi::FT_Pos};
use gl::types::*;

use crate::opengl::camera::Camera;

use super::shader::Shader;

#[derive(PartialEq, Clone, Copy)]
pub enum TextJustification {
    Left,
    Center
}

pub struct Character {
    texture_id: u32, // glyph texture ID
    size: Vector2<i32>, // size of glyph
    pub bearing: Vector2<i32>, // offset from baseline to left/top of glyph
    advance: FT_Pos // offset to advance to next glyph
}

pub struct TextRenderer {
    char_cache: HashMap<usize, Character>, // cached glyph data calculated at construction
    vao: GLuint, // text rendering vertex attributes
    vbo: GLuint, // text rendering buffer
    vao3d: GLuint, // text rendering vertex attributes for 3d rendering
    vbo3d: GLuint, // text rendering buffer for 3d rendering
    shader: Shader, // shader for text rendering
    shader3d: Shader, // shader for text rendering
    screen_width: u32, // screen width
    screen_height: u32 // and screen height for calculating orthographic matrix
}

impl TextRenderer {
    pub unsafe fn new(screen_width: u32, screen_height: u32, font_face: &str) -> TextRenderer {
        // init freetype library
        let lib = Library::init().unwrap();
        
        // load a font face
        let font = lib.new_face(font_face, 0).unwrap();

        // font size
        font.set_pixel_sizes(0, 20).unwrap();

        // disable byte-alignment restriction
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        
        // load shader
        let shader = Shader::new("assets/shaders/text/text_vertex.vert", "assets/shaders/text/text_fragment.frag");
        let shader3d = Shader::new("assets/shaders/text/text_vertex3d.vert", "assets/shaders/text/text_fragment.frag");

        // generate vertex array object
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // allocate empty vertex buffer object for text rendering
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            6 * 4 * std::mem::size_of::<GLfloat>() as isize,
            ptr::null(),
            gl::DYNAMIC_DRAW
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0, 
            4, 
            gl::FLOAT, 
            gl::FALSE, 
            4 * std::mem::size_of::<GLfloat>() as i32, 
            ptr::null()
        );

        // generate vertex array object
        let mut vao3d = 0;
        gl::GenVertexArrays(1, &mut vao3d);
        gl::BindVertexArray(vao3d);

        // allocate empty vertex buffer object for text rendering
        let mut vbo3d = 0;
        gl::GenBuffers(1, &mut vbo3d);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo3d);
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            6 * 5 * std::mem::size_of::<GLfloat>() as isize,
            ptr::null(),
            gl::DYNAMIC_DRAW
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0, 
            3, 
            gl::FLOAT, 
            gl::FALSE, 
            5 * std::mem::size_of::<GLfloat>() as i32, 
            ptr::null()
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1, 
            2, 
            gl::FLOAT, 
            gl::FALSE, 
            5 * std::mem::size_of::<GLfloat>() as i32, 
            (3 * std::mem::size_of::<GLfloat>()) as *const c_void
        );

        // best practice to reset to defaults
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        let mut char_cache: HashMap<usize, Character> = HashMap::new();
        for c in 0..128 {
            // load character
            font.load_char(c, freetype::face::LoadFlag::RENDER)
                .expect(format!("Freetype: failed to load glyph with code '{}'", c).as_str());

            let glyph = font.glyph();
            let bitmap = glyph.bitmap();

            // generate texture object and bind to state to edit
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            
            // add glyph texture data
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as i32,
                bitmap.width(),
                bitmap.rows(),
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                if bitmap.buffer().len() == 0 {
                    ptr::null()
                } else {
                    bitmap.buffer().as_ptr() as *const c_void
                }
            );

            // glyph texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            // add character object
            let character = Character {
                texture_id: texture,
                size: vec2(bitmap.width(), bitmap.rows()),
                bearing: vec2(glyph.bitmap_left(), glyph.bitmap_top()),
                advance: glyph.advance().x
            };
            char_cache.insert(c, character);
        }
        TextRenderer { char_cache, vao, vbo, vao3d, vbo3d, shader, shader3d, screen_width, screen_height }
    }

    pub unsafe fn render_text(&self, text: &str, x: f32, y: f32, scale: f32, color: Vector3<f32>, justification: TextJustification) {
        self.render_text_with_mat(text, x, y, scale, color, SquareMatrix::identity(), justification);
    }

    pub fn get_char(&self, c: char) -> &Character {
        &self.char_cache[&(c as usize)]
    }

    pub fn calc_width(&self, string: &str, scale: f32) -> f32 {
        let mut w = 0.0;
        for c in string.bytes() {
            let ch = &self.char_cache[&(c as usize)];
            w += ((ch.advance >> 6) as f32) * scale;
        } 
        w
    }

    pub unsafe fn render_text3d(&self, camera: &Camera, text: &str, mut x: f32, y: f32, z: f32, scale: f32, color: Vector3<f32>, justification: TextJustification) {
        self.shader3d.use_program();
        self.shader3d.set_mat4("view", camera.get_view());
        self.shader3d.set_mat4("projection", camera.get_projection());
        self.shader3d.set_vec3("textColor", color);

        self.shader3d.set_mat4("model", SquareMatrix::identity());
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindVertexArray(self.vao3d);
    
        if justification == TextJustification::Center {
            let w = self.calc_width(text, scale);
            x = x - w / 2.0;
        }

        for c in text.bytes() {
            let ch = &self.char_cache[&(c as usize)];
    
            let x_pos = x + (ch.bearing.x as f32) * scale;
            let y_pos = match c {
                // hacky solution to properly adjust '-' and '^' respectively
                39 => y + (ch.bearing.y as f32 / 2.0),
                45 => y + (ch.bearing.y as f32 / 2.0),
                94 => y + ch.bearing.y as f32 / 4.0,
                _ => y
            };
    
            let w = (ch.size.x as f32) * scale;
            let h = (ch.size.y as f32) * scale;
    
            // generate vertices for charatcer
            let vertices: [[f32; 5]; 6] = [
                [ x_pos,     y_pos + h, z, 0.0, 0.0 ],
                [ x_pos,     y_pos,     z, 0.0, 1.0 ],
                [ x_pos + w, y_pos,     z, 1.0, 1.0 ],
    
                [ x_pos,     y_pos + h, z, 0.0, 0.0 ],
                [ x_pos + w, y_pos,     z, 1.0, 1.0 ],
                [ x_pos + w, y_pos + h, z, 1.0, 0.0 ]
            ];
    
            // render glyph texture over quad
            gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
    
            // update VBO memory
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo3d);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, std::mem::size_of_val(&vertices) as isize, vertices.as_ptr() as *const c_void);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    
            // render quad
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
    
            x += ((ch.advance >> 6) as f32) * scale;
        }
        gl::BindVertexArray(0);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    pub unsafe fn render_text_with_mat(&self, text: &str, mut x: f32, y: f32, scale: f32, color: Vector3<f32>, model: Matrix4<f32>, justification: TextJustification) {
        self.shader.use_program();
        self.shader.set_mat4("projection", ortho(0.0, self.screen_width as f32, 0.0, self.screen_height as f32, -1.0, 100.0));
        self.shader.set_vec3("textColor", color);
        self.shader.set_mat4("model", model);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindVertexArray(self.vao);
    
        if justification == TextJustification::Center {
            let w = self.calc_width(text, scale);
            x = x - w / 2.0;
        }

        for c in text.bytes() {
            let ch = &self.char_cache[&(c as usize)];
    
            let x_pos = x + (ch.bearing.x as f32) * scale;
            let y_pos = match c {
                // hacky solution to properly adjust '-' and '^' respectively
                39 => y + (ch.bearing.y as f32 / 2.0),
                45 => y + (ch.bearing.y as f32 / 2.0),
                94 => y + ch.bearing.y as f32 / 4.0,
                _ => y
            };
    
            let w = (ch.size.x as f32) * scale;
            let h = (ch.size.y as f32) * scale;
    
            // generate vertices for charatcer
            let vertices: [[f32; 4]; 6] = [
                [ x_pos,     y_pos + h,  0.0, 0.0 ],
                [ x_pos,     y_pos,      0.0, 1.0 ],
                [ x_pos + w, y_pos,      1.0, 1.0 ],
    
                [ x_pos,     y_pos + h,  0.0, 0.0 ],
                [ x_pos + w, y_pos,      1.0, 1.0 ],
                [ x_pos + w, y_pos + h,  1.0, 0.0 ]
            ];
    
            // render glyph texture over quad
            gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
    
            // update VBO memory
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, std::mem::size_of_val(&vertices) as isize, vertices.as_ptr() as *const c_void);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    
            // render quad
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
    
            x += ((ch.advance >> 6) as f32) * scale;
        }
        gl::BindVertexArray(0);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
}