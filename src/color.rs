extern crate image;

use vector::Vector3;
use image::{Rgb, Rgba};

pub fn vector3_to_rgb(color: Vector3) -> Rgb<u8> {
    image::Rgb([
       (255.0 * saturate(color.x)) as u8,
       (255.0 * saturate(color.y)) as u8,
       (255.0 * saturate(color.z)) as u8,
    ])
}

pub fn rgba_to_vector3(color: Rgba<u8>) -> Vector3 {
    Vector3::new(
        color.data[0] as f64 / 255.0,
        color.data[1] as f64 / 255.0,
        color.data[2] as f64 / 255.0,
    )
}

fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}

fn saturate(v: f64) -> f64 {
    clamp(v, 0.0, 1.0)
}