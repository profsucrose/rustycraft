use std::rc::Rc;

use cgmath::Vector3;
use noise::{OpenSimplex};

use crate::models::coord_map::CoordMap;
use super::{block_type::BlockType, chunk::Chunk};

#[derive(Clone)]
pub struct World {
    chunks: CoordMap<Chunk>,
    render_distance: u32,
    simplex: Rc<OpenSimplex>,
    player_chunk_x: i32,
    player_chunk_z: i32,
    mesh: Vec<Rc<Vec<f32>>>
}

// handles world block data and rendering
impl World {
    pub fn new(render_distance: u32) -> World {
        let chunks = CoordMap::new();
        let simplex = Rc::new(OpenSimplex::new());
        
        World { chunks, render_distance, simplex, player_chunk_x: 0, player_chunk_z: 0, mesh: vec![] }
    }

    // pub fn get_meshes(&self) -> Vec<&Vec<f32>> {
    //     let mut mesh = Vec::new();
    //     for z_axis in self.chunks.iter() {
    //         for x_axis in z_axis.1.iter() {
    //             mesh.push(&x_axis.1.mesh);
    //         }
    //     }
    //     mesh
    // }

    pub fn get_world_mesh_from_perspective(&mut self, player_x: i32, player_z: i32) -> &Vec<Rc<Vec<f32>>> {
        let player_chunk_x = player_x / 16;
        let player_chunk_z = player_z / 16;
        if self.mesh.len() > 0 
            && self.player_chunk_x == player_chunk_x 
            && self.player_chunk_z == player_chunk_z {
            return &self.mesh
        }

        self.recalculate_mesh_from_perspective(player_chunk_x, player_chunk_z);

        self.player_chunk_x = player_chunk_x;
        self.player_chunk_z = player_chunk_z;
        
        &self.mesh
    }

    pub fn recalculate_mesh_from_perspective(&mut self, player_chunk_x: i32, player_chunk_z: i32) {
        let mut meshes = Vec::new();
        for x in 0..self.render_distance * 2 {
            let x = (x as i32) - (self.render_distance as i32) + player_chunk_x;
            for z in 0..self.render_distance * 2 {
                let z = (z as i32) - (self.render_distance as i32) + player_chunk_z;
                if (((player_chunk_x - x).pow(2) + (player_chunk_z - z).pow(2)) as f32).sqrt() > self.render_distance as f32 {
                    continue;
                }

                let chunk = self.get_or_insert_chunk(x, z);
                meshes.push(chunk.mesh.clone());
            }
        }

        self.mesh = meshes;
    }

    pub fn get_or_insert_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &Chunk {
        match self.chunks.contains(chunk_x, chunk_z) {
            true => self.chunks.get(chunk_x, chunk_z).unwrap(),
            false => {
                let c = Chunk::new(chunk_x, chunk_z, self.simplex.clone());
                self.chunks.insert(chunk_x, chunk_z, c);
                self.chunks.get(chunk_x, chunk_z).unwrap()
            }
        }
    }

    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut Chunk> {
        match self.chunks.contains(chunk_x, chunk_z) {
            true => self.chunks.get_mut(chunk_x, chunk_z),
            false => None
        }
    }

    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
        match self.chunks.contains(chunk_x, chunk_z) {
            true => self.chunks.get(chunk_x, chunk_z),
            false => None
        }
    }

    pub fn air_at(&self, x: i32, y: i32, z: i32) -> bool {
        let chunk_x = x / 16;
        let chunk_z = z / 16;
        let chunk = self.get_chunk(chunk_x, chunk_z);
        match chunk {
            Some(chunk) => chunk.air_at(x % 16, y, z % 16),
            None => false,
        }
    }

    pub fn get_block(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<BlockType> {
        let chunk_x = world_x / 16;
        let chunk_z = world_z / 16;
        let chunk = self.get_chunk(chunk_x, chunk_z);
        if chunk.is_none() || world_y < 0 {
            return None
        }

        let local_x = (world_x % 16).abs() as usize;
        let local_z = (world_z % 16).abs() as usize;
        Some(chunk.unwrap().block_at(local_x, world_y as usize, local_z))
    }

    pub fn set_block(&mut self, world_x: i32, world_y: i32, world_z: i32, block: BlockType) {
        let chunk_x = world_x / 16;
        let chunk_z = world_z / 16;
        let chunk = self.get_chunk_mut(chunk_x, chunk_z).unwrap();
        //     for x in 0..16 {
        //         for z in 0..16 {
        //             for y in 0..50 {
        //                 chunk.set_block(x, y, z, BlockType::Dirt);
        //             }
        //         }
        //     }
        // }

        // self.get_chunk_mut(chunk_x, chunk_z).unwrap().gen_mesh();

        // for x in 0..16 {
        //     for z in 0..16 {
        //         for y in 0..5 {
        //             println!("{} {} {} = {:?}", x, y, z, self.get_block(x as i32, y as i32, z as i32));
        //         }
        //     }
        // }

        let local_x = (world_x % 16).abs() as usize;
        let local_z = (world_z % 16).abs() as usize;

        println!("Updated {} {} {} to be type {:?}", local_x, world_y, local_z, block);
        chunk.set_block(local_x, world_y as usize, local_z, block);
        println!("{} {} {} = {:?}", local_x, world_y, local_z, self.get_block(world_x, world_y, world_z));
    }

    pub fn raymarch_block(&mut self, position: &Vector3<f32>, direction: &Vector3<f32>) -> Option<(i32, i32, i32)> {
        let mut check_position = *position;
        let mut range = 50;
        let direction = direction * 0.1;
        loop {
            check_position = check_position + direction;
            let x = check_position.x.round() as i32;
            let y = check_position.y.round() as i32;
            let z = check_position.z.round() as i32;
            let block = self.get_block(x, y, z);
            if let Some(block) = block {
                if block != BlockType::Air {
                    println!("Block at {} {} {} is {:?}", x, y, z, block);
                    return Some((x, y, z))
                }
            }

            if range == 0 {
                return None
            }
            range = range - 1;
        }
    }
}