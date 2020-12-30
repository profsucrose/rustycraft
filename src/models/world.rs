use cgmath::{Vector3, vec3};

use super::block::BlockType;

pub struct World {
    blocks: Vec<Vec<Vec<BlockType>>> // 256 x 256 x 10
}

impl World {
    pub fn new() -> World {
        let mut z_axis = Vec::with_capacity(256);
        for _ in 0..50 {
            let mut x_axis = Vec::with_capacity(256);
            for _ in 0..50 {
                let mut y_axis = Vec::with_capacity(10);
                for _ in 0..1 {
                    y_axis.push(BlockType::Dirt);
                }
                x_axis.push(y_axis);
            }
            z_axis.push(x_axis);
        } 

        World { blocks: z_axis }
    }

    pub fn to_model_vectors(&self) -> Vec<Vector3<f32>> {
        let mut result = vec![];
        for x in 0..50 {
            for z in 0..50 {
                for y in 0..self.blocks[x][z].len() {
                    result.push(vec3(x as f32, y as f32, z as f32));
                }
            }
        }
        result
    }
}