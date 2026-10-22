use std::cell::RefCell;
use crate::fixed::Fixed;
use crate::Vector2D;

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
}

thread_local! {
    static SCRATCH: RefCell<Scratch> = RefCell::new(Scratch::new());
}

impl Body {
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