use vector::Vector3;
use config;
use color;

#[allow(dead_code)]
pub enum ToneMappingMode {
    None,
    Reinhard,
}

pub fn execute(color: &Vector3) -> Vector3 {
    match config::TONE_MAPPING_MODE {
        ToneMappingMode::None => none(color),
        ToneMappingMode::Reinhard => reinhard(color, config::TONE_MAPPING_EXPOSURE, config::TONE_MAPPING_WHITE_POINT)
    }
}

fn none(color: &Vector3) -> Vector3 {
    *color
}

fn reinhard(color: &Vector3, exposure: f64, white_point: f64) -> Vector3 {
    let color = *color * exposure;
    let luminance = color::color_to_luminance(&color);
    let white_point = white_point * exposure;
    (color * (luminance / (white_point * white_point) + 1.0) / (luminance + 1.0)).saturate()
}
