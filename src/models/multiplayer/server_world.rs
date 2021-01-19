// world struct for managing and
// parsing world data from a server
use std::{collections::HashSet, fs, sync::Arc};
use std::time::{SystemTime, UNIX_EPOCH};

use cgmath::{Vector3, InnerSpace};
use noise::{OpenSimplex, Seedable};

use crate::models::{core::{block_type::BlockType, coord_map::CoordMap, face::Face}, multiplayer::{rc_message::RustyCraftMessage, server_chunk::ServerChunk}, traits::{game_chunk::GameChunk, game_world::GameWorld}};

use super::server_connection::ServerConnection;

// Vector of Arc (to be thread-safe with server connection listen thread)
// of a tuple of opaque and then transparent block point vertices
type WorldMesh = Vec<Arc<(Vec<f32>, Vec<f32>)>>; 

#[derive(Clone)]
pub struct ServerWorld {
    chunks: CoordMap<ServerChunk>,
    render_distance: u32,
    player_chunk_x: i32,
    player_chunk_z: i32,
    mesh: WorldMesh,
    chunk_fetch_queue: HashSet<(i32, i32)>,
    server_connection: ServerConnection
}

impl GameWorld for ServerWorld {
    fn get_block(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<BlockType> {
        let (chunk_x, chunk_z, local_x, local_z) = self.localize_coords_to_chunk(world_x, world_z);
        let chunk = self.get_chunk(chunk_x, chunk_z);
        if chunk.is_none() || world_y < 0 {
            return None
        }

        let result = Some(chunk.unwrap().block_at(local_x, world_y as usize, local_z));
        result
    }

    fn get_game_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&dyn GameChunk> {
        let result = self.chunks.get(chunk_x, chunk_z);
        match result {
            Some(chunk) => {
                Some(chunk as &dyn GameChunk)
            },
            None => None
        }
    }
}

// handles world block data and rendering
impl ServerWorld {
    pub fn new(render_distance: u32, server_connection: ServerConnection) -> ServerWorld {
        ServerWorld {
            chunks: CoordMap::new(),
            render_distance, 
            player_chunk_x: 0, 
            player_chunk_z: 0, 
            mesh: WorldMesh::new(), 
            chunk_fetch_queue: HashSet::new(), 
            server_connection 
        }
    }

    pub fn get_world_mesh_from_perspective(&mut self, player_x: i32, player_z: i32, force: bool) -> &WorldMesh {
        let player_chunk_x = player_x / 16;
        let player_chunk_z = player_z / 16;
        if !force 
            && self.player_chunk_x == player_chunk_x 
            && self.player_chunk_z == player_chunk_z {
            return &self.mesh
        }

        self.recalculate_mesh_from_perspective(player_chunk_x, player_chunk_z);

        self.player_chunk_x = player_chunk_x;
        self.player_chunk_z = player_chunk_z;
        
        &self.mesh
    }

    pub fn recalculate_mesh_from_player_perspective(&mut self) {
        self.recalculate_mesh_from_perspective(self.player_chunk_x, self.player_chunk_z);
    }

    pub fn recalculate_mesh_from_perspective(&mut self, player_chunk_x: i32, player_chunk_z: i32) {
        let mut meshes = Vec::new();
        let mut chunks_in_view = Vec::new();

        self.chunk_fetch_queue.clear();
        for x in 0..self.render_distance * 2 {
            let x = (x as i32) - (self.render_distance as i32) + player_chunk_x;
            for z in 0..self.render_distance * 2 {
                let z = (z as i32) - (self.render_distance as i32) + player_chunk_z;
                if (((player_chunk_x - x).pow(2) + (player_chunk_z - z).pow(2)) as f32).sqrt() > self.render_distance as f32 {
                    continue;
                }

                let contains_chunk = self.contains_chunk(x, z);
                let contains_chunk_right = self.contains_chunk(x + 1, z);
                let contains_chunk_left = self.contains_chunk(x - 1, z);
                let contains_chunk_front = self.contains_chunk(x, z + 1);
                let contains_chunk_back = self.contains_chunk(x, z - 1);
                if contains_chunk
                        && contains_chunk_right
                        && contains_chunk_left
                        && contains_chunk_front
                        && contains_chunk_back {
                    chunks_in_view.push((x, z));
                } else {
                    if !contains_chunk {
                        self.chunk_fetch_queue.insert((x, z));
                    }

                    if !contains_chunk_right {
                        self.chunk_fetch_queue.insert((x + 1, z));
                    }

                    if !contains_chunk_left {
                        self.chunk_fetch_queue.insert((x - 1, z));
                    }

                    if !contains_chunk_front {
                        self.chunk_fetch_queue.insert((x, z + 1));
                    }

                    if !contains_chunk_back {
                        self.chunk_fetch_queue.insert((x, z - 1));
                    }
                }
            }
        }

        for (x, z) in chunks_in_view.iter() {
            let x = *x;
            let z = *z;

            let mesh;
            let chunk = self.get_chunk(x, z).unwrap();
            if chunk.mesh.0.len() != 0 {
                mesh = chunk.mesh.clone();
            } else {
                let right_chunk = self.get_chunk(x + 1, z).unwrap();
                let left_chunk = self.get_chunk(x - 1, z).unwrap();
                let front_chunk = self.get_chunk(x, z + 1).unwrap();
                let back_chunk = self.get_chunk(x, z - 1).unwrap();
                mesh = chunk.gen_mesh(right_chunk, left_chunk, front_chunk, back_chunk);
            }

            let chunk = self.get_chunk_mut(x, z).unwrap();
            chunk.mesh = mesh;
            meshes.push(chunk.mesh.clone());
        }

        // fetch chunks
        if self.chunk_fetch_queue.len() > 0 {
            self.server_connection.send_message(RustyCraftMessage::GetChunks { coords: self.chunk_fetch_queue.clone().into_iter().collect() })
                .expect("Failed to request batch of chunks")
        }

        self.mesh = meshes;
    }

    pub fn insert_serialized_chunk(&mut self, chunk_x: i32, chunk_z: i32, serialized_chunk: String) {
        let chunk = ServerChunk::from_serialized(serialized_chunk, chunk_x, chunk_z);
        self.chunks.insert(chunk_x, chunk_z, chunk);
        self.chunk_fetch_queue.remove(&(chunk_x, chunk_z));

        // when all chunks requested have been fetched re-render mesh
        if self.chunk_fetch_queue.len() == 0 {
            self.recalculate_mesh_from_perspective(self.player_chunk_x, self.player_chunk_z);
        }
    }

    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ServerChunk> {
        match self.chunks.contains(chunk_x, chunk_z) {
            true => self.chunks.get_mut(chunk_x, chunk_z),
            false => None
        }
    }

    fn contains_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunks.contains(chunk_x, chunk_z)
    }

    pub fn set_block(&mut self, world_x: i32, world_y: i32, world_z: i32, block: BlockType) {
        let (chunk_x, chunk_z, local_x, local_z) = self.localize_coords_to_chunk(world_x, world_z);

        // set block
        {
            let chunk = self.get_chunk_mut(chunk_x, chunk_z).unwrap();
            chunk.set_block(local_x, world_y as usize, local_z, block);
        }

        // update chunk mesh
        self.update_chunk_mesh(chunk_x, chunk_z);
        if local_x == 0 {
            self.update_chunk_mesh(chunk_x - 1, chunk_z)
        } else if local_x == 15 {
            self.update_chunk_mesh(chunk_x + 1, chunk_z)
        } else if local_z == 0 {
            self.update_chunk_mesh(chunk_x, chunk_z - 1)
        } else if local_z == 15 {
            self.update_chunk_mesh(chunk_x, chunk_z + 1)
        }
    }

    fn update_chunk_mesh(&mut self, chunk_x: i32, chunk_z: i32) {
        // assume adjacent chunks exist
        let right_chunk = self.get_chunk(chunk_x + 1, chunk_z).unwrap();
        let left_chunk = self.get_chunk(chunk_x - 1, chunk_z).unwrap();
        let front_chunk = self.get_chunk(chunk_x, chunk_z + 1).unwrap();
        let back_chunk = self.get_chunk(chunk_x, chunk_z - 1).unwrap();
        let chunk = self.get_chunk(chunk_x, chunk_z).unwrap();
        let mesh = chunk.gen_mesh(right_chunk, left_chunk, front_chunk, back_chunk); 

        self.get_chunk_mut(chunk_x, chunk_z).unwrap().mesh = mesh;
    }

    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ServerChunk> {
        self.chunks.get(chunk_x, chunk_z)
    }

    pub fn localize_coords_to_chunk(&self, world_x: i32, world_z: i32) -> (i32, i32, usize, usize) {
        let mut chunk_x = (world_x + if world_x < 0 { 1 } else { 0 }) / 16;
        if world_x < 0 {
            chunk_x -= 1;
        }

        let mut chunk_z = (world_z + if world_z < 0 { 1 } else { 0 }) / 16;
        if world_z < 0 { 
            chunk_z -= 1;
        }

        let local_x = ((chunk_x.abs() * 16 + world_x) % 16).abs() as usize;
        let local_z = ((chunk_z.abs() * 16 + world_z) % 16).abs() as usize;
        (chunk_x, chunk_z, local_x, local_z)
    }

    pub fn raymarch_block(&mut self, position: &Vector3<f32>, direction: &Vector3<f32>) -> Option<((i32, i32, i32), Option<Face>)> {
        let mut check_position = *position;
        let dir: Vector3<f32> = *direction / 10.0;
        let mut range = 250;

        let mut result = Vec::new();
        loop {
            check_position = check_position + dir;
            let x = check_position.x.round() as i32;
            let y = check_position.y.round() as i32;
            let z = check_position.z.round() as i32;
            result.push((x, y, z));

            let block = self.get_block(x, y, z);
            if let Some(block) = block {
                if block != BlockType::Air && block != BlockType::Water {
                    let vector = (*position - (check_position - dir)).normalize();
                    let abs_x = vector.x.abs();
                    let abs_y = vector.y.abs();
                    let abs_z = vector.z.abs();
                    let mut face = None;

                    let mut face_is_x = false;
                    // get cube face from ray direction
                    // negated ray is on x-axis
                    let sign = signum(vector.x);
                    if self.moveable(x + sign, y, z) {
                        face = if vector.x > 0.0 {
                            Some(Face::Right)
                        } else {
                            Some(Face::Left)
                        };
                        face_is_x = true;
                    } 
                    
                    if face.is_none() || abs_y > abs_x { 
                        // negated ray is on y-axis
                        let sign = signum(vector.y);
                        if self.moveable(x, y + sign, z) {
                            face = if vector.y > 0.0 {
                                Some(Face::Top)
                            } else {
                                Some(Face::Bottom)
                            };
                            face_is_x = false;
                        }                        
                    } 
                        
                    let sign = signum(vector.z);
                    if face.is_none() || if face_is_x { abs_z > abs_x } else { abs_z > abs_y } {
                        // negated ray is on z-axis
                        //let sign = signum(vector.z);
                        if self.moveable(x, y, z + sign) {
                            face = if vector.z > 0.0 {
                                Some(Face::Back)
                            } else {
                                Some(Face::Front)
                            }
                        }
                    }

                    return Some(((x, y, z), face));
                }
            }

            if range == 0 {
                return None;
            }
            range = range - 1;
        }
    }
}

fn signum(n: f32) -> i32 {
    if n > 0.0 {
        1
    } else {
        -1
    }
}