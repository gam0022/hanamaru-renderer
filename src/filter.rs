extern crate image;

use image::{ImageBuffer, Rgb};
use config;
use math;
use vector::Vector3;
use color;

pub fn execute(imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
    for _ in 0..config::BILATERAL_FILTER_ITERATION {
        bilateral(imgbuf, config::BILATERAL_FILTER_DIAMETER, config::BILATERAL_FILTER_SIGMA_I, config::BILATERAL_FILTER_SIGMA_S)
    }
}

fn distance(x: u32, y: u32, i: u32, j: u32) -> f64 {
    let dx = x - i;
    let dy = y - j;
    ((dx * dx + dy * dy) as f64).sqrt()
}

fn gaussian(x: f64, sigma: f64) -> f64 {
    (-(x * x) / (2.0 * sigma * sigma)).exp() / (2.0 * config::PI * sigma * sigma)
}

fn bilateral(imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, diameter: u32, sigma_i: f64, sigma_s: f64) {
    let img_clone = imgbuf.clone();
    let width = img_clone.width();
    let height = img_clone.height();
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let mut filtered = Vector3::zero();
        let mut w_p = 0.0;
        let half = diameter / 2;

        let current_lum = (pixel[0] + pixel[1] + pixel[2]) as f64 / 3.0;

        for i in 0..diameter {
            for j in 0..diameter {
                let neighbor_x = math::clamp_u32(x - (half - i), 0, width - 1);
                let neighbor_y = math::clamp_u32(y - (half - j), 0, height - 1);
                let neighbor = img_clone.get_pixel(neighbor_x, neighbor_y);
                let neighbor_lum = (neighbor[0] + neighbor[1] + neighbor[2]) as f64 / 3.0;

                let g_i = gaussian(neighbor_lum - current_lum, sigma_i);
                let g_s = gaussian(distance(x, y, neighbor_x, neighbor_y), sigma_s);
                let w = g_i * g_s;

                filtered += color::rgb_to_color(*neighbor) * w;
                w_p += w;
            }
        }
        *pixel = color::color_to_rgb(filtered / w_p);
    }
}
