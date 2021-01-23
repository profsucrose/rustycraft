#![allow(dead_code)]
use std::collections::{HashMap, HashSet};

use serde_json::{to_string, to_string_pretty};

use crate::models::core::{block_map::BlockMap, block_type::{BlockType, index_to_block}, chunk::{CHUNK_HEIGHT, CHUNK_SIZE}};

type BlocksInMesh = Vec<(usize, usize, usize)>;

fn run_length_encode(to_serialize: &String) -> String {
    let mut result = String::new();
    let length = to_serialize.len();
    let bytes = to_serialize.as_bytes();
    let mut i = 0;
    while i < length {
        let mut count = 1;
        while i < length - 1 && bytes[i] == bytes[i + 1] && count < 63 {
            count += 1;
            i += 1;
        }

        if count > 1 {
            let flagged_count = count | (1 << 6);
            result.push(flagged_count as u8 as char);
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn run_length_decode(serialized: &String) -> String {
    let mut result = String::new();
    let bytes = serialized.as_bytes();
    let mut i = 0;
    while i < serialized.len() {
        let byte = bytes[i];
        // use left-most bit as flag of consecutive byte count
        if (byte >> 6) == 1 {
            let count = byte - 64;
            for _ in 0..count {
                result.push(bytes[i + 1] as char);
            }
            i += 2;
        } else {
            result.push(byte as char);
            i += 1;
        }
    }
    result
}

pub fn from_serialized(serialized: &String) -> (BlocksInMesh, BlockMap) {
    // format (127 as delimiter between layers)
    // 127 <y_greater_than_1> <y mod 127> 16x16 layer grid ...
    // let serialized = run_length_decode(serialized);
    // println!("{:?}", serialized.as_bytes());
    let mut blocks_in_mesh = Vec::new();
    let mut blocks = BlockMap::new();
    let bytes = serialized.as_bytes();
    let mut i = 0;
    let mut y = 0;
    let mut iter_in_layer = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte == 127 {
            y = bytes[i + 1] * 127 + bytes[i + 2];
            iter_in_layer = 0;
            i += 2;
        } else {
            let x = iter_in_layer / 16;
            let z = iter_in_layer % 16;
            blocks_in_mesh.push((x, y as usize, z));
            let block = index_to_block(byte as usize);
            blocks.set(x, y as usize, z, block);
            iter_in_layer += 1;
        }
        i += 1;
    }
    (blocks_in_mesh, blocks)
}

// for calculating compression ratio
pub fn original_serialize(blocks_in_mesh: &BlocksInMesh, blocks: &BlockMap) -> String {
    let mut result = String::new();
    for (x, y, z) in blocks_in_mesh.iter() {
        result.push(*x as u8 as char);
        result.push(*y as u8 as char);
        result.push(*z as u8 as char);
        result.push(blocks.get(*x, *y, *z) as u8 as char);
    }
    result
}

pub fn to_serialized(blocks_in_mesh: &BlocksInMesh, blocks: &BlockMap) -> String {
    // map y to list of x, z and block tuples
    let mut layer_ys = HashSet::new();
    for (_, y, _) in blocks_in_mesh.iter() {
        layer_ys.insert(*y);
    }

    let mut serialized = String::new();
    for y in layer_ys.iter() {
        // use 255 as delimiter, ignored in RLE compression
        serialized.push(127 as u8 as char);
        let y = *y as u8;
        // need two chars to represent 0-255
        let has_127 = if y > 127 { 1u8 } else { 0u8 };
        serialized.push(has_127 as char);
        serialized.push((y % 127) as char);
        for x in 0..16 {
            for z in 0..16 {
                let block = blocks.get(x, y as usize, z);
                serialized.push(block as u8 as char);
            }
        }
    }
    serialized // run_length_encode(&serialized)
}

#[cfg(test)]
mod chunk_tests {
    use std::rc::Rc;

    use noise::OpenSimplex;
    use serde_json::Result;

    use crate::models::{core::chunk::Chunk, multiplayer::rc_message::RustyCraftMessage};

    use super::*;

    #[test]
    fn test_rle() {
        let message = String::from("122333");
        println!("{:?}", run_length_encode(&message).as_bytes());
        assert_eq!(run_length_encode(&message), String::from("1B2C3"));
        assert_eq!(run_length_decode(&run_length_encode(&message)), message);

        let message = String::from("4444444444444444444444444444831875715871835615359671597617965777777");
        assert_eq!(run_length_decode(&run_length_encode(&message)), message); 
    }

    #[test]
    fn test_serialize() {
        let chunk = Chunk::new(0, 0, Rc::new(OpenSimplex::new()), String::from("game_data/worlds/chunk_testing"));
        println!("{}", chunk.blocks_in_mesh.len());
        let serialized = to_serialized(&chunk.blocks_in_mesh, &chunk.blocks);
        let deserialized = Chunk::from(String::new(), serialized, 0, 0);
        for (x, y, z) in chunk.blocks_in_mesh.iter() {
            assert_eq!(chunk.block_at(*x, *y, *z), deserialized.block_at(*x, *y, *z));
        }
    }

    #[test]
    fn test_serialize_and_rle() {
        let chunk = Chunk::new(0, 0, Rc::new(OpenSimplex::new()), String::from("game_data/worlds/chunk_testing"));
        let serialized = to_serialized(&chunk.blocks_in_mesh, &chunk.blocks);
        println!("{:?}", run_length_encode(&serialized).as_bytes());
        // assert_eq!(run_length_decode(&run_length_encode(&serialized)).as_bytes(), serialized.as_bytes());
        // let deserialized = Chunk::from(String::new(), run_length_decode(&serialized), 0, 0);
        // // assert_eq!(&deserialized.blocks_in_mesh, &chunk.blocks_in_mesh);
        // for (x, y, z) in chunk.blocks_in_mesh.iter() {
        //     assert_eq!(chunk.block_at(*x, *y, *z), deserialized.block_at(*x, *y, *z));
        // } 
    }
}