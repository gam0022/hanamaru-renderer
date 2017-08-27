use std::f64;

pub const PI: f64 = f64::consts::PI;
pub const PI2: f64 = 2.0 * PI;

pub const EPS: f64 = 1e-4;
pub const OFFSET: f64 = 1e-4;
pub const INF: f64 = 1e100;

pub const PATHTRACING_BOUNCE_LIMIT: u32 = 10;
pub const PATHTRACING_SAMPLING: u32 = 30;

pub const SUPERSAMPLING: u32 = 2;

pub const GAMMA_FACTOR: f64 = 2.2;

// レイトレ合宿5のレギュレーション用
// https://sites.google.com/site/raytracingcamp5/
pub const REPORT_INTERVAL_SEC: f64 = 30.0;// 30秒ごとに途中結果を出力
pub const TIME_LIMIT_SEC: f64 = 4.3 * (4 * 60 + 33) as f64;// 4分33秒以内に自動で終了
