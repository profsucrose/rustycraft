use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Direction {
    Right,
    Left,
    Forward,
    Backward
}