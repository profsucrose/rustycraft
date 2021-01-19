use super::face::Face;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum BlockType {
    Grass = 0,
    Dirt,
    Log,
    Leaves,
    Stone,
    Air,
    Orange,
    Black,
    DarkOrange,
    Water,
    Sand
}

pub fn index_to_block(index: usize) -> BlockType {
    match index {
        0 => BlockType::Grass,
        1 => BlockType::Dirt,
        2 => BlockType::Log,
        3 => BlockType::Leaves,
        4 => BlockType::Stone,
        5 => BlockType::Air,
        6 => BlockType::Orange,
        7 => BlockType::Black,
        8 => BlockType::DarkOrange,
        9 => BlockType::Water,
        10 => BlockType::Sand,
        _ => panic!("Attempted to convert index {} to BlockType", index)
    }
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
        BlockType::Sand => 12.0,
        BlockType::Air => panic!("Attempted to get block uv for BlockType::Air"),
    }
}