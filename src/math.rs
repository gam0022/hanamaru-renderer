pub fn modulo(a: f64, b: f64) -> f64 {
    let ret = a % b;
    if ret.is_sign_positive() { ret } else { ret + b }
}

pub fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}

pub fn clamp_u32(x: u32, min: u32, max: u32) -> u32 {
    if x < min { min } else if x > max { max } else { x }
}

pub fn saturate(v: f64) -> f64 {
    clamp(v, 0.0, 1.0)
}
