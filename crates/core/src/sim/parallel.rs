use rayon::prelude::*;
use std::sync::Mutex;
use crate::fixed::Fixed;
use super::{accumulate_gravity, SimError, SimulationManager};
use crate::vec::Vector2D;

impl SimulationManager {
    pub fn step_parallel(&mut self, n: u32) -> Result<&Fixed, SimError> {
        for _ in 0..n {
            let overlaps = Mutex::new(Vec::<(usize, usize)>::new());
            let accumulator = Mutex::new(vec![Vector2D::zero(); self.bodies.len()]);

            (0..self.bodies.len()).into_par_iter().for_each(|i| {
                // starts from i til the end
                let mut local_accumulator = vec![Vector2D::zero(); self.bodies.len() - i];
                let mut local_overlaps: Vec<(usize, usize)> = Vec::new();
                let (left, right) = self.bodies.split_at(i + 1);
                let obj1 = &left[i];

                for (j, obj2) in right.iter().enumerate() {
                    let (left, right) = local_accumulator.split_at_mut(j + 1);

                    if accumulate_gravity(
                        &obj1.pos,
                        &obj2.pos,
                        &obj1.radius,
                        &obj2.radius,
                        &obj1.mu,
                        &obj2.mu,
                        &mut left[0],
                        &mut right[0],
                    ) {
                        local_overlaps.push((i, i + 1 + j));
                    }
                }

                let mut overlaps = overlaps.lock().unwrap();
                let mut accumulator = accumulator.lock().unwrap();

                for overlap in local_overlaps {
                    overlaps.push(overlap);
                }

                for (j, val) in local_accumulator.iter().enumerate() {
                    accumulator[j + i] += val;
                }
            });

            let mut overlaps = overlaps.into_inner().unwrap();
            overlaps.sort_unstable();
            for (i, val) in accumulator.lock().unwrap().iter().enumerate() {
                self.bodies[i].a.clone_from(val);
            }
        }

        self.elapsed += &(&self.dt * &Fixed::from_f64(f64::from(n)));

        Ok(&self.elapsed)
    }
}