use crate::{core::block_type::BlockType, traits::game_chunk::GameChunk};
use crate::utils::world_utils::localize_coords_to_chunk;

pub trait GameWorld {
    fn get_block(&self, x: i32, y: i32, z: i32) -> Option<BlockType>;
    fn get_game_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&dyn GameChunk>;
    fn moveable(&self, world_x: i32, world_y: i32, world_z: i32) -> bool {
        let block = self.get_block(world_x, world_y, world_z);
        match block {
            Some(block) => block == BlockType::Air || block == BlockType::Water,
            None => false
        }
    } 
    fn highest_in_column(&self, world_x: i32, world_z: i32) -> Option<usize> {
        let (chunk_x, chunk_z, local_x, local_z) = localize_coords_to_chunk(world_x, world_z);
        let chunk = self.get_game_chunk(chunk_x, chunk_z);
        if chunk.is_none() {
            return None
        }

        Some(chunk.unwrap().highest_in_column(local_x, local_z))
    }
    fn highest_in_column_from_y(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<usize> {
        let (chunk_x, chunk_z, local_x, local_z) = localize_coords_to_chunk(world_x, world_z);
        let chunk = self.get_game_chunk(chunk_x, chunk_z);
        if chunk.is_none() {
            return None
        }

        Some(chunk.unwrap().highest_in_column_from_y(local_x, world_y as usize, local_z)) 
    }
}