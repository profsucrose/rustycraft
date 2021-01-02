use crate::models::{block_face_type::BlockFaceType, face::Face};

use super::block_type::BlockType;

pub struct BlockFace {
    pub vertices: [f32; 30]
}

impl BlockFace {
    pub fn new(block_type: BlockType, face: Face) -> BlockFace {
        let vt_index = block_face_type_to_texture_map_index(block_type_to_block_face_type(block_type, face));
        let vt_left = (vt_index % 6) as f32;
        let vt_right = vt_left + 1.0;
        let vt_top = (vt_index / 6) as f32;
        let vt_bottom = vt_top + 1.0;
        let vertices = match face {
            Face::Top => [
                -0.5,  0.5, -0.5,  vt_left,  vt_bottom,  // bottom-left
                 0.5,  0.5, -0.5,  vt_right, vt_bottom,  // bottom-right
                 0.5,  0.5,  0.5,  vt_right, vt_top,     // top-right
                 0.5,  0.5,  0.5,  vt_right, vt_top,     // top-right
                -0.5,  0.5,  0.5,  vt_left,  vt_top,     // top-left
                -0.5,  0.5, -0.5,  vt_left,  vt_bottom   // bottom-left
            ],
            Face::Bottom => [
                -0.5, -0.5, -0.5,  vt_left,  vt_bottom, // bottom-left
                 0.5, -0.5, -0.5,  vt_right, vt_bottom, // bottom-right
                 0.5, -0.5,  0.5,  vt_right, vt_top, // top-right
                 0.5, -0.5,  0.5,  vt_right, vt_top, // top-right
                -0.5, -0.5,  0.5,  vt_left,  vt_top, // top-left
                -0.5, -0.5, -0.5,  vt_left,  vt_bottom  // bottom-left
            ],
            Face::Left => [
                -0.5,  0.5,  0.5,  vt_left, vt_top, // top-right
                -0.5,  0.5, -0.5,  vt_right, vt_top, // bottom-right
                -0.5, -0.5, -0.5,  vt_right,  vt_bottom, // bottom-left
                -0.5, -0.5, -0.5,  vt_right,  vt_bottom, // bottom-left
                -0.5, -0.5,  0.5,  vt_left, vt_bottom, // bottom-right
                -0.5,  0.5,  0.5,  vt_left, vt_top  // top-right
            ],
            Face::Right => [
                // 0.5,  0.5,  0.5,  vt_right, vt_top, // top-right
                // 0.5,  0.5, -0.5,  vt_left, vt_top, // bottom-right
                // 0.5, -0.5, -0.5,  vt_left,  vt_bottom, // bottom-left
                // 0.5, -0.5, -0.5,  vt_left,  vt_bottom, // bottom-left
                // 0.5, -0.5,  0.5,  vt_right, vt_bottom, // top-left
                // 0.5,  0.5,  0.5,  vt_top, vt_right  // top-right

                0.5,  0.5,  0.5,  vt_left, vt_top, // top-right
                0.5,  0.5, -0.5,  vt_right, vt_top, // bottom-right
                0.5, -0.5, -0.5,  vt_right,  vt_bottom, // bottom-left
                0.5, -0.5, -0.5,  vt_right,  vt_bottom, // bottom-left
                0.5, -0.5,  0.5,  vt_left, vt_bottom, // bottom-right
                0.5,  0.5,  0.5,  vt_left, vt_top  // top-right
            ],
            // turn 90 degrees
            Face::Front => [
                -0.5, -0.5, -0.5,  vt_left,  vt_bottom, // bottom-left
                 0.5, -0.5, -0.5,  vt_right, vt_bottom, // bottom-right
                 0.5,  0.5, -0.5,  vt_right, vt_top,    // top-right
                 0.5,  0.5, -0.5,  vt_right, vt_top,    // top-right
                -0.5,  0.5, -0.5,  vt_left,  vt_top,    // top-left
                -0.5, -0.5, -0.5,  vt_left,  vt_bottom  // bottom-left
            ],
            Face::Back => [
                -0.5, -0.5,  0.5,  vt_left,  vt_bottom, // bottom-left
                 0.5, -0.5,  0.5,  vt_right, vt_bottom, // bottom-right
                 0.5,  0.5,  0.5,  vt_right, vt_top, // top-right
                 0.5,  0.5,  0.5,  vt_right, vt_top, // top-right
                -0.5,  0.5,  0.5,  vt_left,  vt_top, // top-left
                -0.5, -0.5,  0.5,  vt_left,  vt_bottom  // bottom-left
            ]
        };
        BlockFace { vertices }
    }

    pub fn transform(&self, x: f32, y: f32, z: f32) -> BlockFace {
        let mut vertices = self.vertices.clone();
        for (i, v) in vertices.iter_mut().enumerate() {
            *v = match i % 5 {
                0 => *v + x,
                1 => *v + y,
                2 => *v + z,
                _ => *v
            }
        }
        BlockFace { vertices }
    }
}

fn block_type_to_block_face_type(block_face: BlockType, face: Face) -> BlockFaceType {
    match block_face {
        BlockType::Dirt => BlockFaceType::Dirt,
        BlockType::Grass => match face {
            Face::Top => BlockFaceType::Grass,
            Face::Right | Face::Left | Face::Back | Face::Front => BlockFaceType::SideGrass,
            Face::Bottom => BlockFaceType::Dirt
        },
        BlockType::Log => match face {
            Face::Top | Face::Bottom => BlockFaceType::Log,
            _ => BlockFaceType::LogSide
        },
        BlockType::Leaves => BlockFaceType::Leaves,
        BlockType::Stone => BlockFaceType::Stone,
        BlockType::Air => panic!("Attempted to get block face type from BlockType::Air")
    }
}

fn block_face_type_to_texture_map_index(block_face_type: BlockFaceType) -> usize {
    match block_face_type {
        BlockFaceType::Dirt => 0,
        BlockFaceType::Grass => 1,
        BlockFaceType::Stone => 2,
        BlockFaceType::Log => 3,
        BlockFaceType::LogSide => 4,
        BlockFaceType::Leaves => 5,
        BlockFaceType::SideGrass => 6
    }
}