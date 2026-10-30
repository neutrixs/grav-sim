use rayon::prelude::*;
use std::sync::Mutex;
use crate::fixed::Fixed;
use super::{accumulate_gravity, SimError, SimulationManager, integrate_acceleration, fix_collision, integrate_velocity, pair_index};
use crate::vec::Vector2D;

impl SimulationManager {
    pub fn step_parallel(&mut self, n: u32) -> Result<&Fixed, SimError> {
        for _ in 0..n {
            let overlaps = Mutex::new(Vec::<(usize, usize, Fixed)>::new());
            let accumulator = Mutex::new(vec![Vector2D::zero(); self.bodies.len()]);

            (0..self.bodies.len()).into_par_iter().for_each(|i| {
                // starts from i til the end
                let mut local_accumulator = vec![Vector2D::zero(); self.bodies.len() - i];
                let mut local_overlaps: Vec<(usize, usize, Fixed)> = Vec::new();
                // reused for every pair; only moved out (and replaced) when it overlaps
                let mut dist = Fixed::zero();
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
                        &mut dist,
                    ) {
                        local_overlaps.push((i, i + 1 + j, std::mem::replace(&mut dist, Fixed::zero())));
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
            overlaps.sort_unstable_by_key(|o| (o.0, o.1));
            for (i, val) in accumulator.into_inner().unwrap().iter().enumerate() {
                self.bodies[i].a.clone_from(val);
            }

            self.bodies.par_iter_mut().for_each(|body| {
                integrate_acceleration(body, &self.dt);
            });

            // when fixing collision, order matters for consistency
            // therefore as for now, it cannot be parallelized
            for (id_1, id_2, dist) in overlaps.iter() {
                let w = &self.mass_ratio[pair_index(*id_1, *id_2)];
                let (first, second) = self.bodies.split_at_mut(*id_2);
                let a = &mut first[*id_1];
                let b = &mut second[0];

                fix_collision(a, b, dist, w, &self.dt);
            }

            self.bodies.par_iter_mut().for_each(|body| {
                integrate_velocity(body, &self.dt);
            });
        }

        self.elapsed += &(&self.dt * &Fixed::from_f64(f64::from(n)));

        Ok(&self.elapsed)
    }
}