use std::rc::Rc;

use noise::{NoiseFn, OpenSimplex};

use crate::models::{block_face::BlockFace, block_type::BlockType, face::Face};
use rand::prelude::*;

use super::{block_map::BlockMap, block_type::block_to_uv};

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
                let simplex_x = (x as i32 + x_offset) as f32;
                let simplex_z = (z as i32 + z_offset) as f32;
                let noise = gen_heightmap(simplex_x, simplex_z, simplex.clone());
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
                for y in 0..50 {

                    let block = self.blocks.get(x, y, z);
                    if block == BlockType::Air {
                        continue;
                    }

                    let world_x = ((x as i32) + self.x) as f32;
                    let world_y = y as f32;
                    let world_z = ((z as i32) + self.z) as f32;
                    vertices.push(world_x);
                    vertices.push(world_y);
                    vertices.push(world_z);
                  
                    for i in 0..6 {
                        let face = match i {
                            0 => Face::Front,
                            1 => Face::Right,
                            2 => Face::Back,
                            3 => Face::Bottom,
                            4 => Face::Left,
                            5 => Face::Top,
                            _ => panic!("Attempted to convert invalid index to face when setting vertex texture UV indices")
                        };
                        vertices.push(block_to_uv(block, face));
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
        self.gen_mesh();
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


fn gen_heightmap(x: f32, z: f32, simplex: Rc<OpenSimplex>) -> f32 {
    let height = 2.5 * sample_simplex(x / 35.0, z / 35.0, simplex.clone())
    + 0.5 * sample_simplex(x / 10.0, z / 10.0, simplex.clone())
    + 0.25 * sample_simplex(x / 4.0, z / 4.0, simplex.clone());
    height.powf(1.3)
}

fn sample_simplex(x: f32, z: f32, simplex: Rc<OpenSimplex>) -> f32 {
    // noise library returns noise value in range -1.0 to 1.0,
    // so shift over to 0.0 to 1.0 range
    ((simplex.get([x as f64, z as f64]) + 1.0) / 2.0) as f32
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