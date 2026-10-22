pub mod fixed;
pub mod sim;
pub mod body;

use fixed::Fixed;
use std::ops;
use wasm_bindgen::prelude::wasm_bindgen;

pub struct Vector2D {
    x: Fixed,
    y: Fixed,
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

#[wasm_bindgen]
pub struct BodyRetrieved {
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