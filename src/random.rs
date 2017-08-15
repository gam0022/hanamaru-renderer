extern crate rand;

use self::rand::{Rng, ThreadRng};

pub fn get_random(rng: &mut ThreadRng) -> (f64, f64) {
    rng.gen::<(f64, f64)>()
}
