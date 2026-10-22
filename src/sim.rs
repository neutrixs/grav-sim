use wasm_bindgen::prelude::wasm_bindgen;
use crate::Body;

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