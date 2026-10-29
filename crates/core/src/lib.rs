pub mod fixed;
pub mod sim;
pub mod body;
pub mod vec;

use fixed::Fixed;
use vec::Vector2D;
use wasm_bindgen::prelude::wasm_bindgen;

// N reusable Fixed values, for thread_local scratch space:
//     static SCRATCH: RefCell<FixedScratch<3>> = RefCell::new(FixedScratch::new());
//     let [s1, s2, s3] = &mut scratch.0;
// A function holding the borrow must not call another function that borrows
// the same static, or the RefCell panics ("already borrowed").
pub struct FixedScratch<const N: usize>(pub [Fixed; N]);

impl<const N: usize> FixedScratch<N> {
    pub fn new() -> Self {
        FixedScratch(std::array::from_fn(|_| Fixed::zero()))
    }
}

pub struct FixedScratchVec<const N: usize>(pub [Vector2D; N]);
impl<const N: usize> FixedScratchVec<N> {
    pub fn new() -> Self {
        FixedScratchVec(std::array::from_fn(|_| Vector2D::zero()))
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