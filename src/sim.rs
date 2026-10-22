use wasm_bindgen::JsError;
use crate::body::Body;
use wasm_bindgen::prelude::wasm_bindgen;
use std::fmt;
use crate::BodyRetrieved;

#[derive(Debug)]
pub enum SimError {
    Overlap,
    InvalidMass(f64),
    InvalidRadius(f64),
    InvalidPosition(f64, f64),
    InvalidVelocity(f64, f64),
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SimError::Overlap => write!(f, "body overlaps an existing body"),
            SimError::InvalidMass(m) => write!(f, "invalid mass: {m}"),
            SimError::InvalidRadius(r) => write!(f, "invalid radius: {r}"),
            SimError::InvalidPosition(px, py) => write!(f, "invalid position: {px}, {py}"),
            SimError::InvalidVelocity(vx, vy) => write!(f, "invalid velocity: {vx}, {vy}"),
        }
    }
}

impl std::error::Error for SimError {}

#[wasm_bindgen]
pub struct SimulationManager {
    bodies: Vec<Body>,
    next_id: usize,
}

#[wasm_bindgen]
impl SimulationManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SimulationManager {
        SimulationManager { bodies: Vec::new(), next_id: 0 }
    }

    pub fn add_body(
        &mut self,
        pos_x: f64, pos_y: f64,
        v_x: f64, v_y: f64,
        mass: f64, radius: f64,
    ) -> Result<(), JsError> {
        let body = Body::new(pos_x, pos_y, v_x, v_y, mass, radius, self.next_id)?;
        let overlap = self.bodies.iter().any(|other| {
            body.overlap_with(&other, true)
        });

        if overlap {
            return Err(JsError::from(SimError::Overlap));
        }

        self.next_id += 1;
        self.bodies.push(body);

        Ok(())
    }

    pub fn retrieve_all(&self) -> Vec<BodyRetrieved> {
        let mut retrieved: Vec<BodyRetrieved> = vec!();

        for body in self.bodies.iter() {
            retrieved.push(body.retrieve_data());
        }

        retrieved
    }
}