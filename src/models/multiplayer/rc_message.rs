use serde::{Deserialize, Serialize};

use crate::models::{core::block_type::BlockType, multiplayer::direction::Direction};

#[derive(Debug, Serialize, Deserialize)]
pub enum RustyCraftMessage {
    Movement { direction: Direction },
    PlayerJoin { name: String },
    PlayerInit { name: String, x: f32, y: f32, z: f32 },
    PlayerDirection { yaw: f32, pitch: f32 },
    PlayerPosition { x: f32, y: f32, z: f32 },
    SetBlock { block: BlockType, world_x: i32, world_y: i32, world_z: i32 },
    GetChunks { coords: Vec<(i32, i32)> },
    ChatMessage { content: String },

    // echo connection and players id to client
    // to avoid rendering own model and get data
    // for all players currently on the server
    ConnectionData { id: String, players: Vec<(String, String, f32, f32, f32, f32, f32)> /* (x, y, z, yaw, pitch) */ },

    // serialized chunk_blocks in the form of Vec<(usize, usize, usize, usize)>
    // stored as string so serialized chunk blocks can be memoized
    ChunkData { chunks: Vec<(i32, i32, String)> },
    Disconnect
}