#![allow(dead_code)]
use std::{fs, rc::Rc, sync::Arc};

use noise::{NoiseFn, OpenSimplex};

use rand::prelude::*;

use crate::models::{core::{block_map::BlockMap, block_type::{BlockType, block_to_uv, index_to_block}, face::Face}, traits::game_chunk::GameChunk, utils::chunk_utils::from_serialized};

#[derive(Clone)]
pub struct ServerChunk {
    blocks: BlockMap,
    blocks_in_mesh: Vec<(usize, usize, usize)>,
    x: i32,
    z: i32,
    pub mesh: Arc<(Vec<f32>, Vec<f32>)> // cache mesh (Arc to be thread-safe)
}

impl GameChunk for ServerChunk {
    fn get_blocks(&self) -> &BlockMap {
        &self.blocks
    }
}

impl ServerChunk {
    pub fn from_serialized(chunk_data: String, x: i32, z: i32) -> ServerChunk {
        // follows same format as chunk data files 
        // [x] [y] [z] [block_index] 
        // [x1] [y1] [z1] [block_index1] 
        // ...
        let (blocks_in_mesh, blocks) = from_serialized(&chunk_data);
        ServerChunk { blocks, blocks_in_mesh, x: x * 16, z: z * 16, mesh: Arc::new((vec![], vec![])) }
    }

    pub fn gen_mesh(&self, right_chunk: &ServerChunk, left_chunk: &ServerChunk, front_chunk: &ServerChunk, back_chunk: &ServerChunk) -> Arc<(Vec<f32>, Vec<f32>)> {
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

        Arc::new((vertices, water_vertices))
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
    }

    pub fn can_place_at_local_spot(&self, x: i32, y: i32, z: i32, block: BlockType) -> bool {
        if y < 0 {
            return false
        }

        let block_spot = self.blocks.get(x as usize, y as usize, z as usize);
        block_spot == BlockType::Air || (block != BlockType::Water && block_spot == BlockType::Water)
    }

    pub fn can_place_mesh_face_at_block(&self, x: i32, y: i32, z: i32, block: BlockType, right_chunk: &ServerChunk, left_chunk: &ServerChunk, front_chunk: &ServerChunk, back_chunk: &ServerChunk) -> bool {
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
    
    pub fn highest_in_column(&self, x: usize, z: usize) -> usize {
        self.blocks.highest_in_column(x, z)
    }

    pub fn highest_in_column_from_y(&self, x: usize, y: usize, z: usize) -> usize {
        self.blocks.highest_in_column_from_y(x, y, z)
    }
}