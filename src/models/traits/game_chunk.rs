use crate::models::core::{block_map::BlockMap, chunk::CHUNK_HEIGHT};

pub trait GameChunk {
    fn get_blocks(&self) -> &BlockMap;
    fn highest_in_column(&self, x: usize, z: usize) -> usize {
        self.highest_in_column_from_y(x, CHUNK_HEIGHT - 1, z)
    }
    fn highest_in_column_from_y(&self, x: usize, y: usize, z: usize) -> usize {
        self.get_blocks().highest_in_column_from_y(x, y, z)
    }
}