extern crate image;

use config;
use math;
use vector::Vector3;

fn distance(x: u32, y: u32, i: u32, j: u32) -> f64 {
    let dx = x - i;
    let dy = y - j;
    ((dx * dx + dy * dy) as f64).sqrt()
}

fn gaussian(x: f64, sigma: f64) -> f64 {
    (-(x * x) / (2.0 * sigma * sigma)).exp() / (2.0 * config::PI * sigma * sigma)
}

fn xy_to_index(x: u32, y: u32, width: u32) -> usize {
    (y * width + x) as usize
}

fn index_to_xy(index: usize, width: u32) -> (u32, u32) {
    (index as u32 % width, index as u32 / width)
}

pub fn execute(pixel: &Vector3, current: usize, img: &Vec<Vector3>, width: u32, height: u32) -> Vector3 {
    bilateral(pixel, current, img, width, height,
              config::BILATERAL_FILTER_DIAMETER,
              config::BILATERAL_FILTER_SIGMA_I,
              config::BILATERAL_FILTER_SIGMA_S)
}

fn bilateral(pixel: &Vector3, current: usize, img: &Vec<Vector3>, width: u32, height: u32, diameter: u32, sigma_i: f64, sigma_s: f64) -> Vector3 {
    let (x, y) = index_to_xy(current, width);
    let current_sum = pixel.x + pixel.y + pixel.z;
    let sum_scale = 1.0 / 3.0;

    let mut filtered = Vector3::zero();
    let mut w_p = 0.0;
    let half = diameter / 2;

    for i in 0..diameter {
        for j in 0..diameter {
            let neighbor_x = math::clamp_u32(x - (half - i), 0, width - 1);
            let neighbor_y = math::clamp_u32(y - (half - j), 0, height - 1);
            let neighbor = img[xy_to_index(neighbor_x, neighbor_y, width)];
            let neighbor_sum = neighbor.x + neighbor.y + neighbor.z;

            let g_i = gaussian(sum_scale * (neighbor_sum - current_sum), sigma_i);
            let g_s = gaussian(distance(x, y, neighbor_x, neighbor_y), sigma_s);
            let w = g_i * g_s;

            filtered += neighbor * w;
            w_p += w;
        }
    }

    filtered / w_p
}
