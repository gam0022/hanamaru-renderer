use vector::Vector3;
use config;

pub fn none(color: &Vector3) -> Vector3 {
    *color
}

pub fn reinhard(color: &Vector3) -> Vector3 {
    let color = *color * config::TONE_MAPPING_EXPOSURE;
    (color / (color + 1.0)
        * (color / (config::TONE_MAPPING_WHITE_POINT * config::TONE_MAPPING_WHITE_POINT) + 1.0)
    ).saturate()
}
