pub fn modulo(a: f64, b: f64) -> f64 {
    let ret = a % b;
    if ret.is_sign_positive() { ret } else { ret + b }
}
