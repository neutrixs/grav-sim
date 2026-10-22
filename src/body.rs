use crate::Vector2D;
use crate::fixed::Fixed;
use crate::sim::SimError;
use std::cell::RefCell;

struct Scratch {
    s1: Fixed,
    s2: Fixed,
    s3: Fixed,
}

impl Scratch {
    fn new() -> Scratch {
        Scratch {
            s1: Fixed::from_f64(0.),
            s2: Fixed::from_f64(0.),
            s3: Fixed::from_f64(0.),
        }
    }
}

pub struct Body {
    pos: Vector2D,
    v: Vector2D,
    a: Vector2D,
    mass: Fixed,
    radius: Fixed,
    id: usize,
}

thread_local! {
    static SCRATCH: RefCell<Scratch> = RefCell::new(Scratch::new());
}

impl Body {
    pub fn new(
        pos_x: f64,
        pos_y: f64,
        v_x: f64,
        v_y: f64,
        mass: f64,
        radius: f64,
        id: usize
    ) -> Result<Body, SimError> {
        if !(mass.is_finite() && mass > 0.0) {
            return Err(SimError::InvalidMass(mass))
        }

        if !(radius.is_finite() && radius > 0.0) {
            return Err(SimError::InvalidRadius(radius))
        }

        if !(pos_x.is_finite() && pos_y.is_finite()) {
            return Err(SimError::InvalidPosition(pos_x, pos_y))
        }

        if !(v_x.is_finite() && v_y.is_finite()) {
            return Err(SimError::InvalidVelocity(v_x, v_y))
        }

        Ok(Body {
            pos: Vector2D {
                x: Fixed::from_f64(pos_x),
                y: Fixed::from_f64(pos_y),
            },
            v: Vector2D {
                x: Fixed::from_f64(v_x),
                y: Fixed::from_f64(v_y),
            },
            a: Vector2D {
                x: Fixed::from_f64(0.),
                y: Fixed::from_f64(0.),
            },
            mass: Fixed::from_f64(mass),
            radius: Fixed::from_f64(radius),
            id,
        })
    }

    pub fn overlap_with(&self, other: &Body, touch_is_overlap: bool) -> bool {
        SCRATCH.with(|scratch| {
            let s = &mut *scratch.borrow_mut();
            let s1 = &mut s.s1;
            let s2 = &mut s.s2;
            let s3 = &mut s.s3;

            s1.clone_from(&self.radius);
            *s1 += &other.radius;
            s1.square_assign();

            // (x2-x1)^2
            s2.clone_from(&other.pos.x);
            *s2 -= &self.pos.x;
            s2.square_assign();

            // (y2-y1)^2
            s3.clone_from(&other.pos.y);
            *s3 -= &self.pos.y;
            s3.square_assign();

            // (x2-x1)^2 + (y2-y1)^2
            *s2 += &s3;

            if touch_is_overlap {
                s2 <= s1
            } else {
                s2 < s1
            }
       })
    }
}