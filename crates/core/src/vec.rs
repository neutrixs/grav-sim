use std::cell::RefCell;
use crate::fixed::Fixed;
use std::ops;
use crate::FixedScratch;

thread_local! {
    static SCRATCH: RefCell<FixedScratch<1>> = RefCell::new(FixedScratch::new());
}

pub struct Vector2D {
    pub x: Fixed,
    pub y: Fixed,
}

impl ops::Add<&Vector2D> for &Vector2D {
    type Output = Vector2D;

    fn add(self, rhs: &Vector2D) -> Vector2D {
        Vector2D { x: &self.x + &rhs.x, y: &self.y + &rhs.y }
    }
}

impl ops::AddAssign<&Vector2D> for Vector2D {
    fn add_assign(&mut self, rhs: &Vector2D) {
        self.x += &rhs.x;
        self.y += &rhs.y;
    }
}

impl ops::Sub<&Vector2D> for &Vector2D {
    type Output = Vector2D;

    fn sub(self, rhs: &Vector2D) -> Vector2D {
        Vector2D { x: &self.x - &rhs.x, y: &self.y - &rhs.y }
    }
}

impl ops::SubAssign<&Vector2D> for Vector2D {
    fn sub_assign(&mut self, rhs: &Vector2D) {
        self.x -= &rhs.x;
        self.y -= &rhs.y;
    }
}

// vector × scalar
impl ops::Mul<&Fixed> for &Vector2D {
    type Output = Vector2D;

    fn mul(self, rhs: &Fixed) -> Vector2D {
        Vector2D { x: &self.x * rhs, y: &self.y * rhs }
    }
}

impl ops::MulAssign<&Fixed> for Vector2D {
    fn mul_assign(&mut self, rhs: &Fixed) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

// vector / scalar
impl ops::Div<&Fixed> for &Vector2D {
    type Output = Vector2D;

    fn div(self, rhs: &Fixed) -> Vector2D {
        Vector2D { x: &self.x / rhs, y: &self.y / rhs }
    }
}

impl ops::DivAssign<&Fixed> for Vector2D {
    fn div_assign(&mut self, rhs: &Fixed) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Vector2D {
    pub fn zero() -> Vector2D {
        Vector2D {
            x: Fixed::zero(),
            y: Fixed::zero(),
        }
    }

    pub fn set_zero(&mut self) {
        self.x.set_zero();
        self.y.set_zero();
    }

    pub fn negate(&self) -> Vector2D {
        Vector2D {
            x: self.x.negate(),
            y: self.y.negate(),
        }
    }

    pub fn negate_assign(&mut self) {
        self.x.negate_assign();
        self.y.negate_assign();
    }

    pub fn dot(&self, rhs: &Vector2D) -> Fixed {
        SCRATCH.with(|s| {
            let [s1] = &mut s.borrow_mut().0;

            let mut result = Fixed::zero();
            result.clone_from(&self.x);
            result *= &rhs.x;

            s1.clone_from(&self.y);
            *s1 *= &rhs.y;

            result += s1;

            result
        })
    }

    pub fn dot_into(&self, rhs: &Vector2D, target: &mut Fixed) {
        SCRATCH.with(|s| {
            let [s1] = &mut s.borrow_mut().0;

            target.clone_from(&self.x);
            *target *= &rhs.x;

            s1.clone_from(&self.y);
            *s1 *= &rhs.y;

            *target += s1;
        })
    }
}

impl Clone for Vector2D {
    fn clone(&self) -> Vector2D {
        Vector2D { x: self.x.clone(), y: self.y.clone() }
    }

    fn clone_from(&mut self, source: &Vector2D) {
        self.x.clone_from(&source.x);
        self.y.clone_from(&source.y);
    }
}
