use std::{collections::HashMap, ffi::c_void, ptr::null};

use cgmath::vec2;
use freetype::{Face, Library};
use gl::types::*;

use crate::models::character::Character;

pub unsafe fn gen_character_map(font_face: &Face) -> HashMap<usize, Character> {
    let mut map: HashMap<usize, Character> = HashMap::new();
    for c in 0..128 {
        println!("Adding {} with code {} to char map", c as u8 as char, c);
        font_face.load_char(c, freetype::face::LoadFlag::RENDER)
            .expect(format!("Freetype: failed to load glyph with code '{}'", c).as_str());

        println!("Getting glyph");
        let glyph = font_face.glyph();
        let bitmap = glyph.bitmap();
        println!("Loading texture");

        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        
        println!("Buffer size is {} x {} with len {}", bitmap.width(), bitmap.rows(), bitmap.buffer().len());
        //println!("Setting texture image data, len {}, first element {:?} width {}, height: {}", bitmap.buffer().len(), bitmap.buffer()[0] as *const c_void, bitmap.width(), bitmap.rows());
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
                null()
            } else {
                bitmap.buffer().as_ptr() as *const c_void
            }
        );
        println!("Successfully bound texture data");

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let character = Character {
            texture_id: texture,
            size: vec2(bitmap.width(), bitmap.rows()),
            bearing: vec2(glyph.bitmap_left(), glyph.bitmap_top()),
            advance: glyph.advance().x
        };
        map.insert(c, character);
    }
    map
}