#![allow(dead_code)]
use std::collections::HashSet;

use crate::core::{block_map::BlockMap, block_type::index_to_block};

type BlocksInMesh = Vec<(usize, usize, usize)>;

pub fn from_serialized(serialized: &String) -> (BlocksInMesh, BlockMap) {
    // format (127 as delimiter between layers)
    // 127 <y_greater_than_1> <y mod 127> 16x16 layer grid ...
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
            blocks.set(x, y as usize, z, block.unwrap());
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