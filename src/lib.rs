pub mod fixed;
pub mod sim;
pub mod body;

use fixed::Fixed;
use wasm_bindgen::prelude::wasm_bindgen;

pub struct Vector2D {
    x: Fixed,
    y: Fixed,
}

#[wasm_bindgen]
pub struct BodyRetrieved {
    pub id: usize,
    pub pos_x: f64,
    pub pos_y: f64,
    pub v_x: f64,
    pub v_y: f64,
    pub a_x: f64,
    pub a_y: f64,
    pub mass: f64,
    pub radius: f64,
}