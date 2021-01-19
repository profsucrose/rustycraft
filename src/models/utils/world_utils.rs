pub fn localize_coords_to_chunk(world_x: i32, world_z: i32) -> (i32, i32, usize, usize) {
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