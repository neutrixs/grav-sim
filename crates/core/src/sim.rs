use std::cell::RefCell;
use crate::body::Body;
use std::fmt;
use std::sync::LazyLock;
use crate::{FixedScratchVec};
use crate::fixed::Fixed;
use crate::FixedScratch;
use crate::vec::Vector2D;

#[cfg(feature = "parallel")]
mod parallel;

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
    // w = m2/(m1+m2) for every pair, see fix_collision for the math.
    // Indexed with pair_index(i, j). Masses never change, so each value is
    // computed once, when the second body of the pair is added.
    mass_ratio: Vec<Fixed>,
}

// Index of pair (i, j), i < j, in a per-pair table.
// Pairs are grouped by their larger index j:
//
//   j=1: (0,1)
//   j=2: (0,2) (1,2)
//   j=3: (0,3) (1,3) (2,3)
//
// Before group j there are 1 + 2 + ... + (j-1) = j(j-1)/2 pairs, and i is the
// position inside the group. Grouping by j (not i) keeps the table append-only:
// adding body j only adds group j, at the end.
fn pair_index(i: usize, j: usize) -> usize {
    j * (j - 1) / 2 + i
}

impl SimulationManager {
    pub fn new(dt: Fixed) -> Result<Self, SimError> {
        let zero = Fixed::zero();
        if dt <= zero {
            return Err(SimError::InvalidDt(dt.to_f64()));
        }

        Ok(SimulationManager {
            bodies: Vec::new(),
            next_id: 0,
            dt,
            elapsed: Fixed::zero(),
            mass_ratio: Vec::new(),
        })
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

        // group j of mass_ratio: the new body is j (the larger index, m2),
        // every existing body is i (m1)
        for other in self.bodies.iter() {
            // w = m2/(m1+m2)
            let mut w = body.mass.clone();
            w /= &(&other.mass + &body.mass);
            self.mass_ratio.push(w);
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
        let mut overlaps: Vec<(usize, usize, Fixed)> = Vec::new();

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
                // reused for every pair; only moved out (and replaced) when it overlaps
                let mut dist = Fixed::zero();

                for (j, obj2)in right.iter_mut().enumerate() {
                    if accumulate_gravity(
                        &obj1.pos,
                        &obj2.pos,
                        &obj1.radius,
                        &obj2.radius,
                        &obj1.mu,
                        &obj2.mu,
                        &mut obj1.a,
                        &mut obj2.a,
                        &mut dist,
                    ) {
                        overlaps.push((i, i + 1 + j, std::mem::replace(&mut dist, Fixed::zero())))
                    }
                }
            }

            // acceleration is now settled, integrate their acceleration
            for body in self.bodies.iter_mut() {
                integrate_acceleration(body, &self.dt);
            }

            for (id_1, id_2, dist) in overlaps.iter() {
                let w = &self.mass_ratio[pair_index(*id_1, *id_2)];
                let (first, second) = self.bodies.split_at_mut(*id_2);
                let a = &mut first[*id_1];
                let b = &mut second[0];

                fix_collision(a, b, dist, w, &self.dt);
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

// Collision impulse, applied along the normal n = d/|d| (d points from a to b):
//
//   k = β/dt · max(pen - slop, 0) - (v2-v1)·n
//   J = k · m1m2/(m1+m2)            (m1m2/(m1+m2) is the "reduced mass")
//   a.v -= J/m1 · n
//   b.v += J/m2 · n
//
// J itself is never needed, only J/m1 and J/m2, and the masses cancel:
//
//   J/m1 = k · m1m2/(m1+m2) / m1 = k · m2/(m1+m2) = k · w
//   J/m2 = k · m1m2/(m1+m2) / m2 = k · m1/(m1+m2) = k · (1 - w)
//
//   where w = m2/(m1+m2) (cached per pair), and 1 - w = m1/(m1+m2) because
//   m1/(m1+m2) + m2/(m1+m2) = (m1+m2)/(m1+m2) = 1
//
// So no division by mass is left here. k·(1 - w) is computed as k - k·w: then
// the two parts add up to exactly k even after truncation, so the momentum given
// to b is exactly the momentum taken from a.
//
// Clamping: the reduced mass is always > 0, so J and k have the same sign and
// max(J, 0) = 0 exactly when max(k, 0) = 0. The clamp is done on k.
fn fix_collision(a: &mut Body, b: &mut Body, distance: &Fixed, w: &Fixed, dt: &Fixed) {
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

        // s2: pen (r1 + r2 - |d|)
        s2.clone_from(&a.radius);
        *s2 += &b.radius;
        *s2 -= distance;

        // s2: pen - slop (overlap_allowed)
        *s2 -= &*OVERLAP_ALLOWED;

        // s2, s4: max(pen - slop, 0)
        s4.set_zero();
        let max_val = s2.max(s4);

        // s1: beta/dt * max(pen - slop, 0)
        *s1 *= max_val;

        // sv3: n hat (d/|d|)
        sv3.clone_from(sv2);
        *sv3 /= distance;

        // s2: (v2-v1)*n
        sv1.dot_into(sv3, s2);

        // s1: k = beta/dt * max(pen - slop, 0) - (v2-v1)*n
        *s1 -= s2;

        // s1, s3: max(k, 0)
        s3.set_zero();
        let max_val = s1.max(s3);

        // we can now calculate the new velocity
        // for body a

        // s2: J/m1 = max(k, 0) * w
        s2.clone_from(&max_val);
        *s2 *= w;

        // sv1: J/m1 * n
        sv1.clone_from(sv3);
        *sv1 *= &s2;

        a.v -= &sv1;

        // now for body b
        // s4: J/m2 = max(k, 0) * (1 - w) = max(k, 0) - J/m1
        s4.clone_from(max_val);
        *s4 -= &s2;

        // sv1: J/m2 * n
        sv1.clone_from(sv3);
        *sv1 *= &s4;

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
fn accumulate_gravity(
    pos_a: &Vector2D,
    pos_b: &Vector2D,
    radius_a: &Fixed,
    radius_b: &Fixed,
    mu_a: &Fixed,
    mu_b: &Fixed,
    target_a: &mut Vector2D,
    target_b: &mut Vector2D,
    target_d: &mut Fixed,
) -> bool {
    SCRATCH_VEC.with(|sv| SCRATCH.with(|s|{
        let [sv1, sv2, _] = &mut sv.borrow_mut().0;
        let [s1, s2, s3, s4] = &mut s.borrow_mut().0;

        // sv1: distance a->b (d)
        sv1.clone_from(pos_b);
        *sv1 -= pos_a;

        // s1: |d|^2
        sv1.dot_into(&sv1, s1);
        // s2: |d|
        s2.clone_from(s1);
        s2.sqrt_assign();
        target_d.clone_from(s2);

        // s1: |d|^2 * |d| = |d|^3
        *s1 *= s2;

        // let's say there's just no effect if they're exactly at the same place
        if s1.is_zero() {
            return false;
        }

        s3.clone_from(&radius_a);
        *s3 += radius_b;
        let is_overlap = s2 < s3;

        // s4: G*m/|d|^3 (scalar: 1 division instead of 1 per component)
        s4.clone_from(mu_b);
        *s4 /= &s1;

        // sv2: G*m/|d|^3 * d
        sv2.clone_from(sv1);
        *sv2 *= &s4;

        *target_a += &sv2;

        // body A done, now do the same for body B (d points the other way)
        s4.clone_from(mu_a);
        *s4 /= &s1;

        sv2.clone_from(sv1);
        *sv2 *= &s4;

        *target_b -= &sv2;

        is_overlap
    }))
}