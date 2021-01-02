use std::{collections::HashMap, ffi::c_void, ptr};
use cgmath::{Vector2, Vector3, ortho, vec2};
use freetype::Library;
use gl::types::*;

use crate::models::shader::Shader;

pub struct Character {
    pub texture_id: u32, // glyph texture ID
    pub size: Vector2<i32>, // size of glyph
    pub bearing: Vector2<i32>, // offset from baseline to left/top of glyph
    pub advance: i64 // offset to advance to next glyph
}

pub struct TextRenderer {
    char_cache: HashMap<usize, Character>, // cached glyph data calculated at construction
    vao: GLuint, // text rendering vertex attributes
    vbo: GLuint, // text rendering buffer
    shader: Shader, // shader for text rendering
    screen_width: u32, // screen width
    screen_height: u32 // and screen height for calculating orthographic matrix
}

impl TextRenderer {
    pub unsafe fn new(screen_width: u32, screen_height: u32, font_face: &str, text_vertex_shader: &str, text_fragment_shader: &str) -> TextRenderer {
        // init freetype library
        let lib = Library::init().unwrap();
        
        // load a font face
        let font = lib.new_face(font_face, 0).unwrap();

        // font size
        font.set_pixel_sizes(0, 20).unwrap();

        // disable byte-alignment restriction
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        
        // load shader
        let shader = Shader::new(text_vertex_shader, text_fragment_shader);

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
        TextRenderer { char_cache, vao, vbo, shader, screen_width, screen_height }
    }

    pub unsafe fn render_text(&self, text: &str, mut x: f32, y: f32, scale: f32, color: Vector3<f32>) {
        self.shader.use_program();
        self.shader.set_mat4("projection", ortho(0.0, self.screen_width as f32, 0.0, self.screen_height as f32, -1.0, 100.0));
        self.shader.set_vec3("textColor", color);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindVertexArray(self.vao);
    
        for c in text.bytes() {
            let ch = &self.char_cache[&(c as usize)];
    
            let x_pos = x + (ch.bearing.x as f32) * scale;
            let y_pos = match c {
                // hacky solution to properly adjust '-' and '^' respectively
                45 => y + (ch.bearing.y as f32 / 2.0),
                94 => y + ch.bearing.y as f32 / 4.0,
                _ => y // + (ch.bearing.y as f32) * scale;
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