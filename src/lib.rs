pub mod fixed;
pub mod sim;
pub mod body;

use fixed::Fixed;
use std::ops;
use wasm_bindgen::prelude::wasm_bindgen;

// N reusable Fixed values, for thread_local scratch space:
//     static SCRATCH: RefCell<FixedScratch<3>> = RefCell::new(FixedScratch::new());
//     let [s1, s2, s3] = &mut scratch.0;
// A function holding the borrow must not call another function that borrows
// the same static, or the RefCell panics ("already borrowed").
pub struct FixedScratch<const N: usize>(pub [Fixed; N]);

impl<const N: usize> FixedScratch<N> {
    pub fn new() -> Self {
        FixedScratch(std::array::from_fn(|_| Fixed::from_f64(0.)))
    }
}

pub struct FixedScratchVec<const N: usize>(pub [Vector2D; N]);
impl<const N: usize> FixedScratchVec<N> {
    pub fn new() -> Self {
        FixedScratchVec(std::array::from_fn(|_| Vector2D {
            x: Fixed::from_f64(0.),
            y: Fixed::from_f64(0.),
        }))
    }
}

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

impl Clone for Vector2D {
    fn clone(&self) -> Vector2D {
        Vector2D { x: self.x.clone(), y: self.y.clone() }
    }

    fn clone_from(&mut self, source: &Vector2D) {
        self.x.clone_from(&source.x);
        self.y.clone_from(&source.y);
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