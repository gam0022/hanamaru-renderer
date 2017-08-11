use std::f64;

pub const PI: f64 = f64::consts::PI;
pub const PI2: f64 = 2.0 * PI;

pub const EPS: f64 = 1e-4;
pub const OFFSET: f64 = 1e-2;
pub const INF: f64 = 1e100;

pub const DEBUG_BOUNCE_LIMIT: u32 = 3;
pub const PATHTRACING_BOUNCE_LIMIT: u32 = 10;
pub const PATHTRACING_SAMPLING: u32 = 30;

pub const SUPERSAMPLING: u32 = 2;

pub const GAMMA_FACTOR: f64 = 2.2;
