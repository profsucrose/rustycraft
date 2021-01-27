use noise::{NoiseFn, OpenSimplex};

pub fn sample(x: f32, z: f32, simplex: OpenSimplex) -> f32 {
    // noise library returns noise value in range -1.0 to 1.0,
    // so shift over to 0.0 to 1.0 range
    ((simplex.get([x as f64, z as f64]) + 1.0) / 2.0) as f32
}