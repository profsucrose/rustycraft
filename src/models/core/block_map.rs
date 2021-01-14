use super::block_type::BlockType;

use super::chunk::{CHUNK_HEIGHT, CHUNK_SIZE};

// x by y by z 3-dimensional map using C arrays
#[derive(Clone)]
pub struct BlockMap {
    map: [[[BlockType; CHUNK_HEIGHT]; CHUNK_SIZE]; CHUNK_SIZE]
}

impl BlockMap {
    pub fn new() -> BlockMap {
        let map = [[[BlockType::Air; CHUNK_HEIGHT]; CHUNK_SIZE]; CHUNK_SIZE];
        BlockMap { map }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.map[x][z][y]
    }

    pub fn highest_in_column(&self, x: usize, z: usize) -> usize {
        for i in 1..CHUNK_HEIGHT {
            let y = CHUNK_HEIGHT - i;
            if self.get(x, y, z) != BlockType::Air {
                return y
            }
        }
        0
    }

    pub fn highest_in_column_from_y(&self, x: usize, height: usize, z: usize) -> usize {
        for i in 1..height + 1 {
            let y = height - i;
            let block = self.get(x, y, z);
            if block != BlockType::Air && block != BlockType::Water {
                return y
            }
        }
        0 
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        if x >= CHUNK_SIZE {
            panic!(format!("Segfault, attempted to read map at invalid x: {}", x))
        }

        if y >= CHUNK_HEIGHT {
            panic!(format!("Segfault, attempted to read map at invalid y: {}", y))
        }

        if z >= CHUNK_SIZE {
            panic!(format!("Segfault, attempted to read map at invalid z: {}", z))
        }

        self.map[x][z][y] = block;
    }
}