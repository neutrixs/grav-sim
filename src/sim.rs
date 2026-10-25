use std::cell::RefCell;
use wasm_bindgen::JsError;
use crate::body::Body;
use wasm_bindgen::prelude::wasm_bindgen;
use std::fmt;
use crate::{BodyRetrieved, FixedScratchVec};
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
    static SCRATCH: RefCell<FixedScratch<4>> = RefCell::new(FixedScratch::new());
    static SCRATCH_VEC: RefCell<FixedScratchVec<2>> = RefCell::new(FixedScratchVec::new());
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
        Ok(SimulationManager { bodies: Vec::new(), next_id: 0, dt, elapsed: Fixed::zero() })
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
            // on every step, acceleration must be reset to 0 otherwise it'll keep accumulating
            // from gravity (not how it works)
            for body in self.bodies.iter_mut() {
                body.a.set_zero();
            }

            for i in 0..self.bodies.len() {
                let (left, right) = self.bodies.split_at_mut(i + 1);
                let obj1 = &mut left[i];

                for obj2 in right.iter_mut() {
                    accumulate_gravity(obj1, obj2);

                    //TODO: checks for collision
                }
            }

            // acceleration is now settled, integrate their acceleration
            for body in self.bodies.iter_mut() {
                integrate_acceleration(body, &self.dt);
            }

            //TODO: for every collision, adjust their velocity

            // at this point, every body's gravitational interaction with each other has been integrated
            // as velocity
            for body in self.bodies.iter_mut() {
                integrate_velocity(body, &self.dt);
            }
        }

        let elapsed_since_step = &self.dt * &Fixed::from_f64(f64::from(n));
        self.elapsed += &elapsed_since_step;

        Ok(self.elapsed.to_f64())
    }
}

fn integrate_acceleration(body: &mut Body, dt: &Fixed) {
    SCRATCH_VEC.with(|sv| {
        let [sv1, _] = &mut sv.borrow_mut().0;

        sv1.clone_from(&body.a);
        *sv1 *= dt;

        body.v += sv1;
    })
}

fn integrate_velocity(body: &mut Body, dt: &Fixed) {
    SCRATCH_VEC.with(|sv| {
        let [sv1,_] = &mut sv.borrow_mut().0;

        sv1.clone_from(&body.v);
        *sv1 *= &dt;

        body.pos += &sv1;
    })
}

fn accumulate_gravity(body_a: &mut Body, body_b: &mut Body) {
    SCRATCH_VEC.with(|sv| SCRATCH.with(|s|{
        let [sv1,sv2] = &mut sv.borrow_mut().0;
        let [s1, s2, _, _] = &mut s.borrow_mut().0;

        // sv1: distance a->b (d)
        sv1.clone_from(&body_b.pos);
        *sv1 -= &body_a.pos;

        // s1: |d|^2
        sv1.dot_into(&sv1, s1);
        // s2: |d|
        s2.clone_from(s1);
        s2.sqrt_assign();

        // s1: |d|^2 * |d| = |d|^3
        *s1 *= s2;

        // let's say there's just no effect if they're exactly at the same place
        if s1.is_zero() {
            return;
        }

        // sv2: G*m*d/|d|^3
        sv2.clone_from(sv1);
        *sv2 *= &body_b.mu;
        *sv2 /= &s1;

        body_a.a += &sv2;

        // body A done, now do the same for body B
        sv1.negate_assign();
        sv2.clone_from(sv1);
        *sv2 *= &body_a.mu;
        *sv2 /= &s1;

        body_b.a += &sv2;
    }))
}