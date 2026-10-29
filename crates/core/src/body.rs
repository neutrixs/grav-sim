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
    pub pos: Vector2D,
    pub v: Vector2D,
    pub a: Vector2D,
    pub mass: Fixed,
    pub radius: Fixed,
    pub id: usize,
    pub mu: Fixed,
}

thread_local! {
    static SCRATCH: RefCell<FixedScratch<3>> = RefCell::new(FixedScratch::new());
}

impl Body {
    pub fn new(
        pos: Vector2D,
        v: Vector2D,
        mass: Fixed,
        radius: Fixed,
        id: usize
    ) -> Result<Body, SimError> {
        let zero = Fixed::zero();

        if mass <= zero {
            return Err(SimError::InvalidMass(mass.to_f64()))
        }

        if radius <= zero {
            return Err(SimError::InvalidRadius(radius.to_f64()))
        }

        let mu = &mass * &*G;

        Ok(Body {
            pos,
            v,
            a: Vector2D::zero(),
            radius,
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
}