use serde::{Deserialize, Serialize};

use crate::models::{core::block_type::BlockType, multiplayer::direction::Direction};

#[derive(Debug, Serialize, Deserialize)]
pub enum RustyCraftMessage {
    Movement { direction: Direction },
    PlayerRename { name: String },
    PlayerMouseMove { x_offset: f32, z_offset: f32 },
    SetBlock { block: BlockType, world_x: i32, world_y: i32, world_z: i32 },
    GetChunks { coords: Vec<(i32, i32)> },
    
    // serialized chunk_blocks in the form of Vec<(usize, usize, usize, usize)>
    // stored as string so serialized chunk blocks can be memoized
    ChunkData { chunks: Vec<(i32, i32, String)> },
    Disconnect
}