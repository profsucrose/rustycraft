use std::rc::Rc;

use noise::{NoiseFn, OpenSimplex};

use crate::models::{block_face::BlockFace, block_type::BlockType, face::Face};
use rand::prelude::*;

use super::block_map::BlockMap;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

#[derive(Clone)]
pub struct Chunk {
    blocks: BlockMap,
    x: i32,
    z: i32,
    pub mesh: Rc<Vec<f32>> // cache mesh
}

impl Chunk {
    pub fn new(x_offset: i32, z_offset: i32, simplex: Rc<OpenSimplex>) -> Chunk {
        let amplitude = 5.0;
        let mut blocks = BlockMap::new();
        let x_offset = x_offset * 16;
        let z_offset = z_offset * 16;
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let simplex_x = (x as i32 + x_offset) as f64;
                let simplex_z = (z as i32 + z_offset) as f64;
                let noise = simplex.get([simplex_x / 10.0, simplex_z / 10.0]); // octave_simplex(simplex_x / 10.0, simplex_z / 10.0, 10, 1.0, simplex.clone());
                let height = ((noise + 1.0) * amplitude) as usize;
                for y in 0..height {
                    let distance_to_top = height - y;
                    let block = match distance_to_top {
                        1 => BlockType::Grass,
                        2 | 3 => BlockType::Dirt,
                        _ => BlockType::Stone
                    };
                    blocks.set(x, y, z, block);
                }
            }
        } 

        // tree generation logic (hacked together, refactor later)
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < 0.5 {
            let x = (rng.gen::<f32>() * 11.0) as usize + 3;
            let z = (rng.gen::<f32>() * 11.0) as usize + 3;
            let top = blocks.highest_in_column(x, z);
            blocks.set(x, top, z, BlockType::Log);
            blocks.set(x, top + 1, z, BlockType::Log);
            blocks.set(x, top + 2, z, BlockType::Log);
            blocks.set(x, top + 3, z, BlockType::Log);
            blocks.set(x + 1, top + 3, z, BlockType::Leaves);
            blocks.set(x - 1, top + 3, z, BlockType::Leaves);
            blocks.set(x, top + 3, z + 1, BlockType::Leaves);
            blocks.set(x, top + 3, z - 1, BlockType::Leaves);
            blocks.set(x, top + 4, z, BlockType::Leaves);
        }

        let mut chunk = Chunk { blocks, x: x_offset, z: z_offset, mesh: Rc::new(vec![]) };
        chunk.gen_mesh();
        chunk
    }

    fn gen_mesh(&mut self) -> Rc<Vec<f32>> {
        let mut vertices = Vec::new();
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..256 {
                    let block = self.blocks.get(x, y, z);
                    if block == BlockType::Air {
                        continue;
                    }

                    let x = x as i32;
                    let y = y as i32;
                    let z = z as i32;
                    let world_x = x + self.x;
                    let world_y = y as f32;
                    let world_z = z + self.z;
                    if self.air_at(x + 1, y, z) {
                        let face = BlockFace::new(block, Face::Right);
                        vertices.append(&mut face.transform(world_x as f32, world_y as f32, world_z as f32).vertices.to_vec());
                    }

                    if self.air_at(x - 1, y, z) {
                        let face = BlockFace::new(block, Face::Left);
                        vertices.append(&mut face.transform(world_x as f32, world_y as f32, world_z as f32).vertices.to_vec());
                    }

                    if self.air_at(x, y + 1, z) {
                        let face = BlockFace::new(block, Face::Top);
                        vertices.append(&mut face.transform(world_x as f32, world_y as f32, world_z as f32).vertices.to_vec());
                    }

                    if self.air_at(x, y - 1, z) {
                        let face = BlockFace::new(block, Face::Bottom);
                        vertices.append(&mut face.transform(world_x as f32, world_y as f32, world_z as f32).vertices.to_vec());
                    }

                    if self.air_at(x, y, z + 1) {
                        let face = BlockFace::new(block, Face::Back);
                        vertices.append(&mut face.transform(world_x as f32, world_y as f32, world_z as f32).vertices.to_vec());
                    }
                    
                    if self.air_at(x, y, z - 1) {
                        let face = BlockFace::new(block, Face::Front);
                        vertices.append(&mut face.transform(world_x as f32, world_y as f32, world_z as f32).vertices.to_vec());
                    }
                }
            }
        }
        self.mesh = Rc::new(vertices);
        self.mesh.clone()
    }

    pub fn block_at(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks.get(x, y, z)
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        self.blocks.set(x, y, z, block);
    }

    pub fn air_at(&self, x: i32, y: i32, z: i32) -> bool {
        if y < 2 {
            return false
        }

        if x < 0 || x >= CHUNK_SIZE as i32
            || y >= CHUNK_HEIGHT as i32
            || z < 0 || z >= CHUNK_SIZE as i32 {
            return true
        }

        self.blocks.get(x as usize, y as usize, z as usize) == BlockType::Air
    }
}


fn octave_simplex(x: f32, z: f32, octaves: i32, persistence: f32, simplex: Rc<OpenSimplex>) -> f32 {
    let mut total = 0f32;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0; 
    for _ in 0..octaves {
        total += (simplex.get([(x * frequency) as f64, (z * frequency) as f64]) * (amplitude as f64)) as f32;
        
        max_value += amplitude;
        
        amplitude *= persistence;
        frequency *= 2.0;
    }
    total / max_value
}