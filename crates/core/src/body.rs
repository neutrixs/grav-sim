use crate::BodyRetrieved;
use crate::vec::Vector2D;
use crate::fixed::Fixed;
use crate::FixedScratch;
use crate::sim::SimError;
use std::cell::RefCell;
use std::sync::LazyLock;

static G: LazyLock<Fixed> = LazyLock::new(|| {
    if let Ok(g) = Fixed::from_ratio(667430.0, 1e16) {
        return g;
    }

    unreachable!()
});

pub struct Body {
    pub(crate) pos: Vector2D,
    pub(crate) v: Vector2D,
    pub(crate) a: Vector2D,
    pub(crate) mass: Fixed,
    pub(crate) radius: Fixed,
    pub(crate) id: usize,
    pub(crate) mu: Fixed,
}

thread_local! {
    static SCRATCH: RefCell<FixedScratch<3>> = RefCell::new(FixedScratch::new());
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

        let mass = Fixed::from_f64(mass);
        let mu = &mass * &*G;

        Ok(Body {
            pos: Vector2D {
                x: Fixed::from_f64(pos_x),
                y: Fixed::from_f64(pos_y),
            },
            v: Vector2D {
                x: Fixed::from_f64(v_x),
                y: Fixed::from_f64(v_y),
            },
            a: Vector2D::zero(),
            radius: Fixed::from_f64(radius),
            mass,
            id,
            mu,
        })
    }

    pub fn overlap_with(&self, other: &Body, touch_is_overlap: bool) -> bool {
        SCRATCH.with(|scratch| {
            let s = &mut *scratch.borrow_mut();
            let [s1, s2, s3] = &mut s.0;

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

    pub fn retrieve_data(&self) -> BodyRetrieved {
        BodyRetrieved {
            id: self.id,
            pos_x: self.pos.x.to_f64(),
            pos_y: self.pos.y.to_f64(),
            v_x: self.v.x.to_f64(),
            v_y: self.v.y.to_f64(),
            a_x: self.a.x.to_f64(),
            a_y: self.a.y.to_f64(),
            radius: self.radius.to_f64(),
            mass: self.mass.to_f64(),
        }
    }
}