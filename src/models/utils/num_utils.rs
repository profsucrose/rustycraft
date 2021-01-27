pub fn distance(x1: i32, z1: i32, x2: i32, z2: i32) -> f32 {
    (((x1 - x2).pow(2) + (z1 - z2).pow(2)) as f32).sqrt()
}