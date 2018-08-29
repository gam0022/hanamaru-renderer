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

// 解像度とサンプリング数
//pub const RESOLUTION: (u32, u32, u32) = (1280, 720, 10);// 16:9 HD 921,600 pixel
//pub const RESOLUTION: (u32, u32, u32) = (800, 600, 10);// 4:3 SVGA 480,000 pixel
//pub const RESOLUTION: (u32, u32, u32) = (1280, 960, 1000);// 4:3 960p 1,228,800 pixel
//pub const RESOLUTION: (u32, u32, u32) = (1440, 1080, 1000);// 4:3 1080p 1,555,200 pixel
//pub const RESOLUTION: (u32, u32, u32) = (2592, 3625, 1000);// B5 + とんぼ(2508 + 42 *2, 3541 + 42 *2)
//pub const RESOLUTION: (u32, u32, u32) = (2592/4, 3625/4, 100);// B5 + とんぼ(2508 + 42 *2, 3541 + 42 *2)
//pub const RESOLUTION: (u32, u32, u32) = (1024, 1024, 1000);
//pub const RESOLUTION: (u32, u32, u32) = (1920 / 4, 1080 / 4, 1);// 16:9 Half FHD
pub const RESOLUTION: (u32, u32, u32) = (1920, 1080, 1000);// 16:9 FHD 2,073,600 pixel

// レイトレ合宿6のレギュレーション用
// https://sites.google.com/site/raytracingcamp6/
pub const REPORT_INTERVAL_SEC: f64 = 15.0;// 15秒ごとに途中結果を出力
// pub const TIME_LIMIT_SEC: f64 = (123 * 10000) as f64;// 123秒以内に自動で終了
pub const TIME_LIMIT_SEC: f64 = (1047) as f64;// 開発環境と本番環境の性能差を考慮した制限時間

// Tone Mapping
pub const TONE_MAPPING_MODE: ToneMappingMode = ToneMappingMode::Reinhard;
pub const TONE_MAPPING_EXPOSURE: f64 = 2.0;
pub const TONE_MAPPING_WHITE_POINT: f64 = 100.0;

// Denoising - Bilateral Fileter
pub const BILATERAL_FILTER_ITERATION: u32 = 0;
pub const BILATERAL_FILTER_DIAMETER: u32 = 3;
pub const BILATERAL_FILTER_SIGMA_I: f64 = 1.0;// これを無限大にすると Gaussian Blur となる
pub const BILATERAL_FILTER_SIGMA_S: f64 = 16.0;
