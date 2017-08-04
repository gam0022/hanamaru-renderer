extern crate rand;

use self::rand::{Rng, ThreadRng};

pub fn get_random(rng: &mut ThreadRng) -> (f64, f64) {
    (rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0))
}