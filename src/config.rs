use std::f64;
use tonemap::ToneMappingMode;

pub const PI: f64 = f64::consts::PI;
pub const PI2: f64 = 2.0 * PI;

pub const EPS: f64 = 1e-4;
pub const OFFSET: f64 = 1e-4;
pub const INF: f64 = 1e100;

pub const GAMMA_FACTOR: f64 = 2.2;

pub const SUPERSAMPLING: u32 = 2;
pub const PATHTRACING_BOUNCE_LIMIT: u32 = 10;

// レイトレ合宿5のレギュレーション用
// https://sites.google.com/site/raytracingcamp5/
pub const REPORT_INTERVAL_SEC: f64 = 30.0;// 30秒ごとに途中結果を出力
pub const TIME_LIMIT_SEC: f64 = (4*60*60) as f64;// 4分33秒以内に自動で終了

// Tone Mapping
pub const TONE_MAPPING_MODE: ToneMappingMode = ToneMappingMode::Reinhard;
pub const TONE_MAPPING_EXPOSURE: f64 = 2.0;
pub const TONE_MAPPING_WHITE_POINT: f64 = 100.0;

// Denoising - Bilateral Fileter
pub const BILATERAL_FILTER_ITERATION: u32 = 1;
pub const BILATERAL_FILTER_DIAMETER: u32 = 3;
pub const BILATERAL_FILTER_SIGMA_I: f64 = 60.0;
pub const BILATERAL_FILTER_SIGMA_S: f64 = 5.0;
