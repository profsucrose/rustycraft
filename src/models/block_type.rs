#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BlockType {
    Grass,
    Dirt,
    Log,
    Leaves,
    Stone,
    Air
}

pub fn block_to_uv(block_face_type: BlockType) -> f32 {
    match block_face_type {
        BlockType::Dirt => 0.0,
        BlockType::Grass => 1.0,
        BlockType::Stone => 2.0,
        BlockType::Log => 3.0,
        BlockType::Leaves => 4.0,
        BlockType::Air => panic!("Attempted to get block uv for BlockType::Air"),
    }
}