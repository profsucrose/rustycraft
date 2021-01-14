use std::rc::Rc;

use noise::{NoiseFn, OpenSimplex};

use rand::prelude::*;

use super::{block_map::BlockMap, block_type::{BlockType, block_to_uv}, face::Face, world::World};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

#[derive(Clone)]
pub struct Chunk {
    blocks: BlockMap,
    blocks_in_mesh: Vec<(usize, usize, usize)>,
    x: i32,
    z: i32,
    pub mesh: Rc<Vec<f32>> // cache mesh
}

impl Chunk {
    pub fn new(x_offset: i32, z_offset: i32, simplex: Rc<OpenSimplex>) -> Chunk {
        let amplitude = 5.0;
        let mut blocks = BlockMap::new();
        let mut blocks_in_mesh = Vec::new();
        let x_offset = x_offset * 16;
        let z_offset = z_offset * 16;
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let simplex_x = (x as i32 + x_offset) as f32;
                let simplex_z = (z as i32 + z_offset) as f32;
                let noise = gen_heightmap(simplex_x, simplex_z, simplex.clone());
                let height = (noise * amplitude) as usize;
                if noise < 0.4 {
                    // if height is low enough make flat water
                    for y in 0..(0.4 * amplitude) as usize - 1 {
                        blocks.set(x, y, z, BlockType::Water);
                        blocks_in_mesh.push((x, y, z));
                    }
                } else {
                    for y in 0..height {
                        let distance_to_top = height - y;
                        let block =
                            if noise < 0.7 {
                                BlockType::Sand
                            } else {
                                match distance_to_top {
                                    1 => BlockType::Grass,
                                    2 | 3 => BlockType::Dirt,
                                    _ => BlockType::Stone
                                }
                            };
                        
                        blocks.set(x, y, z, block);
                        blocks_in_mesh.push((x, y, z));
                    }
                }                
            }
        } 

        // tree generation logic (hacked together, refactor later)
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < 0.9 {
            let x = (rng.gen::<f32>() * 11.0) as usize + 3;
            let z = (rng.gen::<f32>() * 11.0) as usize + 3;
            let top = blocks.highest_in_column(x, z);
            if blocks.get(x, top, z) != BlockType::Water {
                blocks.set(x, top, z, BlockType::Log);
                blocks_in_mesh.push((x, top, z));

                blocks.set(x, top + 1, z, BlockType::Log);
                blocks_in_mesh.push((x, top + 1, z));

                blocks.set(x, top + 2, z, BlockType::Log);
                blocks_in_mesh.push((x, top + 2, z));

                blocks.set(x, top + 3, z, BlockType::Log);
                blocks_in_mesh.push((x, top + 3, z));

                blocks.set(x + 1, top + 3, z, BlockType::Leaves);
                blocks_in_mesh.push((x + 1, top + 3, z));
                
                blocks.set(x - 1, top + 3, z, BlockType::Leaves);
                blocks_in_mesh.push((x - 1, top + 3, z));

                blocks.set(x, top + 3, z + 1, BlockType::Leaves);
                blocks_in_mesh.push((x, top + 3, z + 1));

                blocks.set(x, top + 3, z - 1, BlockType::Leaves);
                blocks_in_mesh.push((x, top + 3, z - 1));

                blocks.set(x, top + 4, z, BlockType::Leaves);
                blocks_in_mesh.push((x, top + 4, z));
            }
        }

        Chunk { blocks, blocks_in_mesh, x: x_offset, z: z_offset, mesh: Rc::new(vec![]) }
    }

    pub fn gen_mesh(&self, right_chunk: &Chunk, left_chunk: &Chunk, front_chunk: &Chunk, back_chunk: &Chunk) -> Rc<Vec<f32>> {
        let mut vertices = Vec::new();
        for (x, y, z) in self.blocks_in_mesh.iter() {
            // let instant = std::time::Instant::now();
            let x = *x;
            let y = *y;
            let z = *z;

            let block = self.blocks.get(x, y, z);
            if block == BlockType::Air {
                continue;
            }

            let x = x as i32;
            let y = y as i32;
            let z = z as i32;
            let faces =
                0
                | if self.air_at_adj_chunks(x, y, z - 1, right_chunk, left_chunk, front_chunk, back_chunk) { 0b10000000 } else { 0 }
                | if self.air_at_adj_chunks(x + 1, y, z, right_chunk, left_chunk, front_chunk, back_chunk) { 0b01000000 } else { 0 }
                | if self.air_at_adj_chunks(x, y, z + 1, right_chunk, left_chunk, front_chunk, back_chunk) { 0b00100000 } else { 0 }
                | if self.air_at_adj_chunks(x, y - 1, z, right_chunk, left_chunk, front_chunk, back_chunk) { 0b00010000 } else { 0 }
                | if self.air_at_adj_chunks(x - 1, y, z, right_chunk, left_chunk, front_chunk, back_chunk) { 0b00001000 } else { 0 }
                | if self.air_at_adj_chunks(x, y + 1, z, right_chunk, left_chunk, front_chunk, back_chunk) { 0b00000100 } else { 0 };

            if faces == 0 {
                continue;
            }

            let world_x = (x + self.x) as f32;
            let world_y = y as f32;
            let world_z = (z + self.z) as f32;
            
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

            vertices.push(faces as f32);
            // println!("Took {:?} to add a block", instant.elapsed());
        }
        //self.mesh = Rc::new(vertices);

        Rc::new(vertices)
    }

    pub fn block_at(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks.get(x, y, z)
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        self.blocks.set(x, y, z, block);
        if block == BlockType::Air {
            for i in 0..self.blocks_in_mesh.len() - 1 {
                if self.blocks_in_mesh[i] == (x, y, z) {
                    self.blocks_in_mesh.remove(i);
                    break;
                }
            }
        } else {
            self.blocks_in_mesh.push((x, y, z));
        }
        //self.gen_mesh();
    }

    pub fn air_at(&self, x: i32, y: i32, z: i32) -> bool {
        if y < 0 {
            return false
        }

        // if x < 0 || x >= CHUNK_SIZE as i32
        //     || y >= CHUNK_HEIGHT as i32
        //     || z < 0 || z >= CHUNK_SIZE as i32 {
        //     return true
        // }

        self.blocks.get(x as usize, y as usize, z as usize) == BlockType::Air
    }

    pub fn air_at_adj_chunks(&self, x: i32, y: i32, z: i32, right_chunk: &Chunk, left_chunk: &Chunk, front_chunk: &Chunk, back_chunk: &Chunk) -> bool {
        if y < 0 {
            return false
        }

        // if outside own chunk fetch edge
        // of respective adjacent chunk
        if x == 16 {
            return right_chunk.air_at(0, y, z);
        } else if x == -1 {
            return left_chunk.air_at(15, y, z);
        } else if z == 16 {
            return front_chunk.air_at(x, y, 0);
        } else if z == -1 {
            return back_chunk.air_at(x, y, 15);
        }

        self.blocks.get(x as usize, y as usize, z as usize) == BlockType::Air
    }
    
    pub fn highest_in_column(&self, x: usize, z: usize) -> usize {
        self.blocks.highest_in_column(x, z)
    }

    pub fn highest_in_column_from_y(&self, x: usize, y: usize, z: usize) -> usize {
        self.blocks.highest_in_column_from_y(x, y, z)
    }
}

fn gen_heightmap(x: f32, z: f32, simplex: Rc<OpenSimplex>) -> f32 {
    // get distance from center
    let nx = x / 4.0 - 0.5;
    let nz = z / 4.0 - 0.5;
    let d = (nx * nx + nz * nz).sqrt() / (0.5 as f32).sqrt(); 

    let height = 5.0 * sample_simplex(x / 35.0, z / 35.0, simplex.clone())
    + 2.0 * sample_simplex(x / 10.0, z / 10.0, simplex.clone())
    + 0.25 * sample_simplex(x / 4.0, z / 4.0, simplex.clone());
    let height = height.powf(1.2);
    (1.0 + height - d.powf(0.5)) / 2.0
}

fn sample_simplex(x: f32, z: f32, simplex: Rc<OpenSimplex>) -> f32 {
    // noise library returns noise value in range -1.0 to 1.0,
    // so shift over to 0.0 to 1.0 range
    ((simplex.get([x as f64, z as f64]) + 1.0) / 2.0) as f32
}