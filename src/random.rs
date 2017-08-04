extern crate rand;

use self::rand::{thread_rng, Rng};

pub fn hash() -> f64 {
    let mut rng = thread_rng();
    rng.gen_range(0.0, 1.0)
}