pub mod fixed;
pub mod sim;
pub mod body;

use wasm_bindgen::prelude::*;
use std::ops::{Add, Mul};
use fixed::Fixed;

pub struct Vector2D {
    x: Fixed,
    y: Fixed,
}

#[wasm_bindgen]
pub struct SimulationManager {
    bodies: Vec<body::Body>,
}

#[wasm_bindgen]
impl SimulationManager {
    pub fn new() -> SimulationManager {
        SimulationManager { bodies: Vec::new() }
    }
}
