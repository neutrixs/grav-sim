use std::cell::RefCell;
use crate::body::Body;
use std::fmt;
use std::sync::LazyLock;
use crate::{FixedScratchVec};
use crate::fixed::Fixed;
use crate::FixedScratch;
use crate::vec::Vector2D;

static BIAS_FACTOR: LazyLock<Fixed> = LazyLock::new(|| {
   Fixed::from_f64(0.125)
});

static OVERLAP_ALLOWED: LazyLock<Fixed> = LazyLock::new(|| {
    Fixed::from_f64(0.125)
});

// TODO: must be a type of Fixed that implements Debug
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

thread_local! {
    static SCRATCH: RefCell<FixedScratch<4>> = RefCell::new(FixedScratch::new());
    static SCRATCH_VEC: RefCell<FixedScratchVec<3>> = RefCell::new(FixedScratchVec::new());
}

pub struct SimulationManager {
    bodies: Vec<Body>,
    next_id: usize,
    dt: Fixed,
    elapsed: Fixed,
}

impl SimulationManager {
    pub fn new(dt: Fixed) -> Result<Self, SimError> {
        let zero = Fixed::zero();
        if dt <= zero {
            return Err(SimError::InvalidDt(dt.to_f64()));
        }

        Ok(SimulationManager { bodies: Vec::new(), next_id: 0, dt, elapsed: Fixed::zero() })
    }

    pub fn add_body(
        &mut self,
        pos: Vector2D,
        v: Vector2D,
        mass: Fixed, radius: Fixed,
    ) -> Result<(), SimError> {
        let body = Body::new(pos, v, mass, radius, self.next_id)?;
        let overlap = self.bodies.iter().any(|other| {
            body.overlap_with(&other, true)
        });

        if overlap {
            return Err(SimError::Overlap);
        }

        self.next_id += 1;
        self.bodies.push(body);

        Ok(())
    }

    pub fn retrieve_all(&self) -> &Vec<Body> {
        &self.bodies
    }

    pub fn dt(&self) -> &Fixed {
        &self.dt
    }

    pub fn set_dt(&mut self, dt: Fixed) -> Result<(), SimError> {
        let zero = Fixed::zero();
        if dt <= zero {
            return Err(SimError::InvalidDt(dt.to_f64()))
        }

        self.dt = dt;
        Ok(())
    }

    pub fn time(&self) -> &Fixed {
        &self.elapsed
    }

    pub fn step(&mut self, n: u32) -> Result<&Fixed, SimError> {
        let mut overlaps: Vec<(usize, usize)> = Vec::new();

        for _ in 0..n {
            overlaps.clear();
            // on every step, acceleration must be reset to 0 otherwise it'll keep accumulating
            // from gravity (not how it works)
            for body in self.bodies.iter_mut() {
                body.a.set_zero();
            }

            for i in 0..self.bodies.len() {
                let (left, right) = self.bodies.split_at_mut(i + 1);
                let obj1 = &mut left[i];

                for (j, obj2)in right.iter_mut().enumerate() {
                    if accumulate_gravity(obj1, obj2) {
                        overlaps.push((i, i + 1 + j))
                    }
                }
            }

            // acceleration is now settled, integrate their acceleration
            for body in self.bodies.iter_mut() {
                integrate_acceleration(body, &self.dt);
            }

            for (id_1, id_2) in overlaps.iter() {
                let (first, second) = self.bodies.split_at_mut(*id_2);
                let a = &mut first[*id_1];
                let b = &mut second[0];

                fix_collision(a, b, &self.dt);
            }

            // at this point, every body's gravitational interaction with each other has been integrated
            // as velocity
            for body in self.bodies.iter_mut() {
                integrate_velocity(body, &self.dt);
            }
        }

        let elapsed_since_step = &self.dt * &Fixed::from_f64(f64::from(n));
        self.elapsed += &elapsed_since_step;

        Ok(&self.elapsed)
    }
}

fn fix_collision(a: &mut Body, b: &mut Body, dt: &Fixed) {
    SCRATCH_VEC.with(|sv| SCRATCH.with(|s| {
        let [s1, s2, s3, s4] = &mut s.borrow_mut().0;
        let [sv1, sv2, sv3] = &mut sv.borrow_mut().0;

        sv1.clone_from(&b.v);
        *sv1 -= &a.v;

        sv2.clone_from(&b.pos);
        *sv2 -= &a.pos;

        // s1: (v2-v1) * d
        sv1.dot_into(sv2, s1);
        s2.set_zero();
        // to save performance, it should not be calculated when the object is moving away from each other
        if s1 > s2 {
            return
        }

        // s1: beta(bias factor) / dt
        s1.clone_from(&*BIAS_FACTOR);
        *s1 /= dt;

        // s3: |d|
        sv2.dot_into(&sv2, s3);
        s3.sqrt_assign();

        // s2: pen (r1 + r2 - |d|)
        s2.clone_from(&a.radius);
        *s2 += &b.radius;
        *s2 -= &s3;

        // s2: pen - slop (overlap_allowed)
        *s2 -= &*OVERLAP_ALLOWED;

        // s2, s4: max(pen - slop, 0)
        s4.set_zero();
        let max_val = s2.max(s4);

        // s1: beta/dt * max(pen - slop, 0)
        *s1 *= max_val;

        // sv3: n hat (d/|d|)
        sv3.clone_from(sv2);
        *sv3 /= s3;

        // s2: (v2-v1)*n
        sv1.dot_into(sv3, s2);

        // s1: beta/dt * max(pen - slop, 0) - (v2-v1)*n
        *s1 -= s2;

        // s2: m1m2
        s2.clone_from(&a.mass);
        *s2 *= &b.mass;

        // s3: m1+m2
        s3.clone_from(&a.mass);
        *s3 += &b.mass;

        // s2: m1m2/(m1+m2)
        *s2 /= &s3;

        // s1: J
        *s1 *= &s2;

        // we can now calculate the new velocity
        // for body a

        // s1, s3: max(J, 0)
        s3.set_zero();
        let max_val = s1.max(s3);

        // s2: max(J, 0)/m1
        s2.clone_from(&max_val);
        *s2 /= &a.mass;

        // sv1: max(J, 0)/m1 * n
        sv1.clone_from(sv3);
        *sv1 *= &s2;

        a.v -= &sv1;

        // now for body b
        // s2: max(J, 0)/m2
        s2.clone_from(max_val);
        *s2 /= &b.mass;

        // sv1: max(J, 0)/m2 * n
        sv1.clone_from(sv3);
        *sv1 *= &s2;

        b.v += &sv1;
    }))
}

fn integrate_acceleration(body: &mut Body, dt: &Fixed) {
    SCRATCH_VEC.with(|sv| {
        let [sv1, _, _] = &mut sv.borrow_mut().0;

        sv1.clone_from(&body.a);
        *sv1 *= dt;

        body.v += sv1;
    })
}

fn integrate_velocity(body: &mut Body, dt: &Fixed) {
    SCRATCH_VEC.with(|sv| {
        let [sv1, _, _] = &mut sv.borrow_mut().0;

        sv1.clone_from(&body.v);
        *sv1 *= &dt;

        body.pos += &sv1;
    })
}

// returns bool of whether they overlap
fn accumulate_gravity(body_a: &mut Body, body_b: &mut Body) -> bool {
    SCRATCH_VEC.with(|sv| SCRATCH.with(|s|{
        let [sv1, sv2, _] = &mut sv.borrow_mut().0;
        let [s1, s2, s3, _] = &mut s.borrow_mut().0;

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
            return false;
        }

        s3.clone_from(&body_a.radius);
        *s3 += &body_b.radius;
        let is_overlap = s2 < s3;

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

        is_overlap
    }))
}