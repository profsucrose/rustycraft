use std::{fs, rc::Rc};
use std::time::{SystemTime, UNIX_EPOCH};

use cgmath::{Vector3, InnerSpace};
use noise::{OpenSimplex, Seedable};

use crate::{traits::{game_chunk::GameChunk, game_world::GameWorld}, utils::{num_utils::distance, world_utils::localize_coords_to_chunk}};

use super::{block_type::BlockType, chunk::{CHUNK_HEIGHT, Chunk}, coord_map::CoordMap, face::Face};

// Vector of Rc of a tuple of opaque and then transparent block point vertices
type WorldMesh = Vec<Rc<(Vec<f32>, Vec<f32>)>>; 

#[derive(Clone)]
pub struct World {
    chunks: CoordMap<Chunk>,
    render_distance: u32,
    simplex: OpenSimplex,
    player_chunk_x: i32,
    player_chunk_z: i32,
    pub save_dir: String,
    mesh: WorldMesh
}

impl GameWorld for World {
    fn get_block(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<BlockType> {
        let (chunk_x, chunk_z, local_x, local_z) = localize_coords_to_chunk(world_x, world_z);
        let chunk = self.get_chunk(chunk_x, chunk_z);
        if chunk.is_none() || world_y < 0 || world_y >= CHUNK_HEIGHT as i32 {
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
impl World {
    pub fn new_with_seed(render_distance: u32, save_dir: &str, seed: u32) -> World {
        // create world directory if it does not exist
        let dir = format!("game_data/worlds/{}/chunks", save_dir);
        fs::create_dir_all(dir.clone()) 
            .expect(format!("Failed to recursively create {}", dir.clone()).as_str());

        let chunks = CoordMap::new();
        let simplex = OpenSimplex::new().set_seed(seed);
        
        let save_dir = format!("game_data/worlds/{}", save_dir);
        World { chunks, render_distance, simplex, player_chunk_x: 0, player_chunk_z: 0, save_dir, mesh: vec![] }
    }

    pub fn new(render_distance: u32, save_dir: &str) -> World {
        let seed_path = format!("game_data/worlds/{}/seed", save_dir);
        let seed = fs::read_to_string(seed_path.clone());
        // read seed from world dir otherwise create
        // one and write to disk
        let seed = match seed {
            Ok(seed) => seed.parse::<u32>().unwrap(),
            Err(_) => {
                let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u32;
                fs::create_dir_all(format!("game_data/worlds/{}", save_dir))
                    .expect("Failed to create world directory");
                fs::write(seed_path.clone(), format!("{}", seed))
                    .expect(format!("Failed to write seed to {}", seed_path).as_str());
                seed
            }
        };
        World::new_with_seed(render_distance, save_dir, seed)
    }

    pub fn get_world_mesh_from_perspective(&mut self, player_x: i32, player_z: i32, force: bool) -> &WorldMesh {
        let player_chunk_x = player_x / 16;
        let player_chunk_z = player_z / 16;
        if !force 
            && self.mesh.len() > 0 
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
        let mut chunks_in_view = Vec::new();
        for x in 0..self.render_distance * 2 {
            let x = (x as i32) - (self.render_distance as i32) + player_chunk_x;
            for z in 0..self.render_distance * 2 {
                let z = (z as i32) - (self.render_distance as i32) + player_chunk_z;
                if distance(player_chunk_x, player_chunk_z, x, z) > self.render_distance as f32 {
                    continue;
                }

                self.get_or_insert_chunk(x, z);
                self.get_or_insert_chunk(x + 1, z);
                self.get_or_insert_chunk(x - 1, z);
                self.get_or_insert_chunk(x, z + 1);
                self.get_or_insert_chunk(x, z - 1);
                chunks_in_view.push((x, z));
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

        self.mesh = meshes;
    }

    pub fn get_or_insert_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &Chunk {
        match self.chunks.contains(chunk_x, chunk_z) {
            true => self.chunks.get(chunk_x, chunk_z).unwrap(),
            false => {
                let c = Chunk::new(chunk_x, chunk_z, self.simplex.clone(), format!("{}/chunks", self.save_dir));
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
        self.chunks.get(chunk_x, chunk_z)
    }

    pub fn set_block(&mut self, world_x: i32, world_y: i32, world_z: i32, block: BlockType) {
        let (chunk_x, chunk_z, local_x, local_z) = localize_coords_to_chunk(world_x, world_z);

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