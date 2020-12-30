use cgmath::Vector2;

pub struct Character {
    pub texture_id: u32, // glyph texture ID
    pub size: Vector2<i32>, // size of glyph
    pub bearing: Vector2<i32>, // offset from baseline to left/top of glyph
    pub advance: i64 // offset to advance to next glyph
}