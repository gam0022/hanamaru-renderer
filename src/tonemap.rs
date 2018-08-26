use vector::Vector3;
use config;

#[allow(dead_code)]
pub enum ToneMappingMode {
    None,
    Reinhard,
}

pub fn execute(color: &Vector3) -> Vector3 {
    match config::TONE_MAPPING_MODE {
        ToneMappingMode::None => none(color),
        ToneMappingMode::Reinhard => reinhard(color)
    }
}

fn none(color: &Vector3) -> Vector3 {
    *color
}

fn reinhard(color: &Vector3) -> Vector3 {
    let color = *color * config::TONE_MAPPING_EXPOSURE;
    (color / (color + 1.0)
        * (color / (config::TONE_MAPPING_WHITE_POINT * config::TONE_MAPPING_WHITE_POINT) + 1.0)
    ).saturate()
}
