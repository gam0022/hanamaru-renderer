extern crate image;

use vector::Vector3;
use image::{Rgb, Rgba};
use math::saturate;
use config;

pub type Color = Vector3;

pub fn color_to_rgb(color: Color) -> Rgb<u8> {
    Rgb([
       (255.0 * saturate(color.x)) as u8,
       (255.0 * saturate(color.y)) as u8,
       (255.0 * saturate(color.z)) as u8,
    ])
}

pub fn rgba_to_color(color: Rgba<u8>) -> Color {
    Color::new(
        color.data[0] as f64 / 255.0,
        color.data[1] as f64 / 255.0,
        color.data[2] as f64 / 255.0,
    )
}

pub fn rgb_to_color(color: Rgb<u8>) -> Color {
    Color::new(
        color.data[0] as f64 / 255.0,
        color.data[1] as f64 / 255.0,
        color.data[2] as f64 / 255.0,
    )
}

fn gamma_to_linear_f64(v: f64) -> f64 {
    v.powf(config::GAMMA_FACTOR)
}

pub fn gamma_to_linear(color: Color) -> Color {
    Color::new(
        gamma_to_linear_f64(color.x),
        gamma_to_linear_f64(color.y),
        gamma_to_linear_f64(color.z),
    )
}

fn linear_to_gamma_f64(v: f64) -> f64 {
    v.powf(config::GAMMA_FACTOR.recip())
}

pub fn linear_to_gamma(color: Color) -> Color {
    Color::new(
        linear_to_gamma_f64(color.x),
        linear_to_gamma_f64(color.y),
        linear_to_gamma_f64(color.z),
    )
}

// https://stackoverflow.com/questions/3018313/algorithm-to-convert-rgb-to-hsv-and-hsv-to-rgb-in-range-0-255-for-both
pub fn hsv_to_rgb(color: Color) -> Color {
    ((hue(color.x) - 1.0) * color.y + 1.0) * color.z
}

fn hue(h: f64) -> Color {
    Color::new(
        saturate((h * 6.0 - 3.0).abs() - 1.0),
        saturate(2.0 - (h * 6.0 - 2.0).abs()),
        saturate(2.0 - (h * 6.0 - 4.0).abs())
    )
}
