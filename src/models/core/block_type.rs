use super::face::Face;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BlockType {
    Grass,
    Dirt,
    Log,
    Leaves,
    Stone,
    Air,
    Orange,
    Black,
    DarkOrange,
    Water
}

pub fn block_to_uv(block_face_type: BlockType, face: Face) -> f32 {
    match block_face_type {
        BlockType::Dirt => 0.0,
        BlockType::Grass => match face {
            Face::Top => 1.0,
            Face::Bottom => 0.0,
            _ => 6.0
        },
        BlockType::Stone => 2.0,
        BlockType::Log => match face {
            Face::Top | Face::Bottom => 3.0,
            _ => 4.0
        },
        BlockType::Leaves => 5.0,
        BlockType::Orange => 8.0,
        BlockType::DarkOrange => 9.0,
        BlockType::Black => 10.0,
        BlockType::Water => 11.0,
        BlockType::Air => panic!("Attempted to get block uv for BlockType::Air"),
    }
}