use crate::body::Body;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct SimulationManager {
    bodies: Vec<Body>,
}

#[wasm_bindgen]
impl SimulationManager {
    pub fn new() -> SimulationManager {
        SimulationManager { bodies: Vec::new() }
    }
}