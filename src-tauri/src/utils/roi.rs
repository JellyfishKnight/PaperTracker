use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roi {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}