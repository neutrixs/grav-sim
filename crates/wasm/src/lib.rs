pub use grav_core::*;
use wasm_bindgen::prelude::*;
use grav_core::fixed::Fixed;
use grav_core::sim::{SimError, SimulationManager};
use grav_core::vec::Vector2D;

#[wasm_bindgen]
pub struct Body {
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

#[wasm_bindgen]
pub struct Sim {
    internal_sim: SimulationManager,
}

#[wasm_bindgen]
impl Sim {
    #[wasm_bindgen(constructor)]
    pub fn new(dt: f64) -> Result<Self, JsError> {
        if !(dt.is_finite() && dt > 0.) {
            return Err(JsError::from(SimError::InvalidDt(dt)))
        }

        let internal_sim = SimulationManager::new(Fixed::from_f64(dt))?;
        Ok(Sim { internal_sim })
    }

    pub fn add_body(
        &mut self,
        pos_x: f64,
        pos_y: f64,
        v_x: f64,
        v_y: f64,
        mass: f64,
        radius: f64,
    ) -> Result<(), JsError> {
        if !pos_x.is_finite() || !pos_y.is_finite() {
            return Err(JsError::from(SimError::InvalidPosition(pos_x, pos_y)))
        }

        if !v_x.is_finite() || !v_y.is_finite() {
            return Err(JsError::from(SimError::InvalidVelocity(v_x, v_y)))
        }

        if !(mass.is_finite() && mass > 0.) {
            return Err(JsError::from(SimError::InvalidMass(mass)));
        }

        if !(radius.is_finite() && radius > 0.) {
            return Err(JsError::from(SimError::InvalidRadius(radius)));
        }

        self.internal_sim.add_body(
            Vector2D { x: Fixed::from_f64(pos_x), y: Fixed::from_f64(pos_y) },
            Vector2D { x: Fixed::from_f64(v_x), y: Fixed::from_f64(v_y) },
            Fixed::from_f64(mass),
            Fixed::from_f64(radius),
        )?;

        Ok(())
    }

    pub fn retrieve(&self) -> Vec<Body> {
        let mut bodies: Vec<Body> = Vec::new();

        for int_body in self.internal_sim.retrieve_all() {
            bodies.push(Body {
                id: int_body.id,
                pos_x: int_body.pos.x.to_f64(),
                pos_y: int_body.pos.y.to_f64(),
                v_x: int_body.v.x.to_f64(),
                v_y: int_body.v.y.to_f64(),
                a_x: int_body.a.x.to_f64(),
                a_y: int_body.a.y.to_f64(),
                mass: int_body.mass.to_f64(),
                radius: int_body.radius.to_f64(),
            })
        }

        bodies
    }

    #[wasm_bindgen(getter)]
    pub fn time(&self) -> f64 {
        self.internal_sim.time().to_f64()
    }

    #[wasm_bindgen(getter)]
    pub fn dt(&self) -> f64 {
        self.internal_sim.dt().to_f64()
    }

    #[wasm_bindgen(setter)]
    pub fn set_dt(&mut self, dt: f64) -> Result<(), JsError> {
        if !(dt.is_finite() && dt > 0.) {
            return Err(JsError::from(SimError::InvalidDt(dt)))
        }

        let new_dt = Fixed::from_f64(dt);
        self.internal_sim.set_dt(new_dt)?;

        Ok(())
    }

    pub fn step(&mut self, n: i32) -> Result<(), JsError> {
        let Ok(n) = u32::try_from(n) else {
            return Err(JsError::from(SimError::InvalidStep(n)))
        };

        self.internal_sim.step(n)?;

        Ok(())
    }
}
