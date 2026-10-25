use crate::fixed::Fixed;
use std::ops;

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
