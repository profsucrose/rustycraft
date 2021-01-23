use std::{fs, rc::Rc};

use noise::{NoiseFn, OpenSimplex};

use rand::prelude::*;

use crate::models::{core::block_type::index_to_block, traits::game_chunk::GameChunk, utils::chunk_utils::{from_serialized, to_serialized}};

use super::{block_map::BlockMap, block_type::{BlockType, block_to_uv}, face::Face, world::World};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

#[derive(Clone)]
pub struct Chunk {
    pub blocks: BlockMap,
    pub blocks_in_mesh: Vec<(usize, usize, usize)>,
    x: i32,
    z: i32,
    save_path: String,
    pub mesh: Rc<(Vec<f32>, Vec<f32>)> // cache mesh
}

impl GameChunk for Chunk {
    fn get_blocks(&self) -> &BlockMap {
        &self.blocks
    }
}

impl Chunk {
    pub fn from(save_path: String, contents: String, x: i32, z: i32) -> Chunk {
        let  (blocks_in_mesh, blocks) = from_serialized(&contents); 
        Chunk { blocks, blocks_in_mesh, x: x * 16, z: z * 16, save_path, mesh: Rc::new((vec![], vec![])) }
    }

    pub fn new(x_offset: i32, z_offset: i32, simplex: Rc<OpenSimplex>, chunk_dir: String) -> Chunk {
        let save_path = format!("{}/{}_{}", chunk_dir, x_offset, z_offset);
        let contents = fs::read_to_string(save_path.clone());
        if let Ok(contents) = contents {
            return Chunk::from(save_path, contents, x_offset, z_offset)
        }

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
                let height = (noise * amplitude) as usize + 3;
                if noise < 0.4 {
                    // if height is low enough make flat water
                    // let sand_level = 3;
                    // for y in 0..sand_level {
                    //     blocks.set(x, y, z, BlockType::Water);
                    // }
                    for y in 0..(0.4 * amplitude) as usize + 2 {
                        if y < height - 1 {
                            blocks.set(x, y, z, BlockType::Sand);
                        } else {
                            blocks.set(x, y, z, BlockType::Water);
                        }
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

        let chunk = Chunk { blocks, blocks_in_mesh, x: x_offset, z: z_offset, save_path, mesh: Rc::new((vec![], vec![])) };
        chunk.save();
        chunk
    }

    pub fn gen_mesh(&self, right_chunk: &Chunk, left_chunk: &Chunk, front_chunk: &Chunk, back_chunk: &Chunk) -> Rc<(Vec<f32>, Vec<f32>)> {
        let mut vertices = Vec::new();
        // water is transparent so is in separate
        // vector to draw after opaque blocks
        let mut water_vertices = Vec::new();
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
                | if self.can_place_mesh_face_at_block(x, y, z - 1, block, right_chunk, left_chunk, front_chunk, back_chunk) { 0b10000000 } else { 0 }
                | if self.can_place_mesh_face_at_block(x + 1, y, z, block, right_chunk, left_chunk, front_chunk, back_chunk) { 0b01000000 } else { 0 }
                | if self.can_place_mesh_face_at_block(x, y, z + 1, block, right_chunk, left_chunk, front_chunk, back_chunk) { 0b00100000 } else { 0 }
                | if self.can_place_mesh_face_at_block(x, y - 1, z, block, right_chunk, left_chunk, front_chunk, back_chunk) { 0b00010000 } else { 0 }
                | if self.can_place_mesh_face_at_block(x - 1, y, z, block, right_chunk, left_chunk, front_chunk, back_chunk) { 0b00001000 } else { 0 }
                | if self.can_place_mesh_face_at_block(x, y + 1, z, block, right_chunk, left_chunk, front_chunk, back_chunk) { 0b00000100 } else { 0 };
                

            if faces == 0 {
                continue;
            }

            let world_x = (x + self.x) as f32;
            let world_y = y as f32;
            let world_z = (z + self.z) as f32;
            
            let vertices_to_push_to = if block == BlockType::Water {
                &mut water_vertices
            }else {
                &mut vertices
            };
            vertices_to_push_to.push(world_x);
            vertices_to_push_to.push(world_y);
            vertices_to_push_to.push(world_z);
            
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
                vertices_to_push_to.push(block_to_uv(block, face));
            }

            vertices_to_push_to.push(faces as f32);
            // println!("Took {:?} to add a block", instant.elapsed());
        }
        //self.mesh = Rc::new(vertices);

        Rc::new((vertices, water_vertices))
    }

    fn save(&self) {
        fs::write(self.save_path.clone(), to_serialized(&self.blocks_in_mesh, &self.blocks))
            .expect(format!("Failed to save chunk to {}", self.save_path.clone()).as_str());
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
        self.save();
        //self.gen_mesh();
    }

    pub fn can_place_at_local_spot(&self, x: i32, y: i32, z: i32, block: BlockType) -> bool {
        if y < 0 {
            return false
        }

        // if x < 0 || x >= CHUNK_SIZE as i32
        //     || y >= CHUNK_HEIGHT as i32
        //     || z < 0 || z >= CHUNK_SIZE as i32 {
        //     return true
        // }

        let block_spot = self.blocks.get(x as usize, y as usize, z as usize);
        block_spot == BlockType::Air || (block != BlockType::Water && block_spot == BlockType::Water)
    }

    pub fn can_place_mesh_face_at_block(&self, x: i32, y: i32, z: i32, block: BlockType, right_chunk: &Chunk, left_chunk: &Chunk, front_chunk: &Chunk, back_chunk: &Chunk) -> bool {
        if y < 0 {
            return false
        }

        // if outside own chunk fetch edge
        // of respective adjacent chunk
        if x == 16 {
            return right_chunk.can_place_at_local_spot(0, y, z, block);
        } else if x == -1 {
            return left_chunk.can_place_at_local_spot(15, y, z, block);
        } else if z == 16 {
            return front_chunk.can_place_at_local_spot(x, y, 0, block);
        } else if z == -1 {
            return back_chunk.can_place_at_local_spot(x, y, 15, block);
        }

        let block_spot = self.blocks.get(x as usize, y as usize, z as usize);
        block_spot == BlockType::Air || (block_spot == BlockType::Water && block != BlockType::Water)
    }
}

fn gen_heightmap(x: f32, z: f32, simplex: Rc<OpenSimplex>) -> f32 {
    // get distance from center
    let nx = x / 5.0 - 0.5;
    let nz = z / 5.0 - 0.5;
    let d = (nx * nx + nz * nz).sqrt() / (0.5 as f32).sqrt(); 

    let height = 5.0 * sample_simplex(x / 35.0, z / 35.0, simplex.clone())
    + 2.0 * sample_simplex(x / 10.0, z / 10.0, simplex.clone())
    + 0.25 * sample_simplex(x / 4.0, z / 4.0, simplex.clone());
    let height = height.powf(1.3);
    (1.0 + height - d.powf(0.5)) / 2.0
}

fn sample_simplex(x: f32, z: f32, simplex: Rc<OpenSimplex>) -> f32 {
    // noise library returns noise value in range -1.0 to 1.0,
    // so shift over to 0.0 to 1.0 range
    ((simplex.get([x as f64, z as f64]) + 1.0) / 2.0) as f32
}