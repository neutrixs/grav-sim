use std::cell::RefCell;
use wasm_bindgen::JsError;
use crate::body::Body;
use wasm_bindgen::prelude::wasm_bindgen;
use std::fmt;
use crate::BodyRetrieved;
use crate::fixed::Fixed;
use crate::FixedScratch;

#[derive(Debug)]
pub enum SimError {
    Overlap,
    InvalidMass(f64),
    InvalidRadius(f64),
    InvalidPosition(f64, f64),
    InvalidVelocity(f64, f64),
    InvalidDt(f64),
    InvalidStep(i32),
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SimError::Overlap => write!(f, "body overlaps an existing body"),
            SimError::InvalidMass(m) => write!(f, "invalid mass: {m}"),
            SimError::InvalidRadius(r) => write!(f, "invalid radius: {r}"),
            SimError::InvalidPosition(px, py) => write!(f, "invalid position: {px}, {py}"),
            SimError::InvalidVelocity(vx, vy) => write!(f, "invalid velocity: {vx}, {vy}"),
            SimError::InvalidDt(dt) => write!(f, "invalid dt: {dt}"),
            SimError::InvalidStep(step) => write!(f, "invalid step: {step}"),
        }
    }
}

impl std::error::Error for SimError {}

// dt must be finite and > 0, and still > 0 after conversion to fixed-point
// (anything below 2^-FRAC_BITS would round down to a zero step).
fn validate_dt(dt: f64) -> Result<Fixed, SimError> {
    if !(dt.is_finite() && dt > 0.0) {
        return Err(SimError::InvalidDt(dt));
    }

    let fixed = Fixed::from_f64(dt);
    if fixed.is_zero() {
        return Err(SimError::InvalidDt(dt));
    }

    Ok(fixed)
}

thread_local! {
    static SCRATCH: RefCell<FixedScratch<4>> = RefCell::new(FixedScratch::new())
}

#[wasm_bindgen]
pub struct SimulationManager {
    bodies: Vec<Body>,
    next_id: usize,
    dt: Fixed,
    elapsed: Fixed,
}

#[wasm_bindgen]
impl SimulationManager {
    #[wasm_bindgen(constructor)]
    pub fn new(dt: f64) -> Result<SimulationManager, JsError> {
        let dt = validate_dt(dt)?;
        Ok(SimulationManager { bodies: Vec::new(), next_id: 0, dt, elapsed: Fixed::from_f64(0.) })
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

    pub fn dt(&self) -> f64 {
        self.dt.to_f64()
    }

    pub fn set_dt(&mut self, dt: f64) -> Result<(), JsError> {
        self.dt = validate_dt(dt)?;
        Ok(())
    }

    pub fn time(&self) -> f64 {
        self.elapsed.to_f64()
    }

    pub fn step(&mut self, n: i32) -> Result<f64, JsError> {
        if n <= 0 {
            return Err(JsError::from(SimError::InvalidStep(n)))
        }

        for _ in 0..n {
            // add code for gravity, acceleration, collision

            //


        }

        let elapsed_since_step = &self.dt * &Fixed::from_f64(f64::from(n));
        self.elapsed += &elapsed_since_step;

        Ok(self.elapsed.to_f64())
    }
}