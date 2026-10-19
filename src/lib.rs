use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}! (from Rust)")
}

#[wasm_bindgen]
pub fn add(a: f64, b: f64) -> f64 {
    a + b
}
