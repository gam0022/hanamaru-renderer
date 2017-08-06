extern crate image;

use image::{DynamicImage, GenericImage};
use std::path::Path;

use vector::Vector3;
use color;

pub struct Texture {
    pub image: DynamicImage,
}

fn clamp(x: u32, min: u32, max: u32) -> u32 {
    if x < min { min } else if x > max { max } else { x }
}

impl Texture {
    pub fn new(path: &str) -> Texture {
        Texture {
            image: image::open(&Path::new(path)).unwrap(),
        }
    }

    pub fn sample_bilinear_0center(&self, u: f64, v: f64) -> Vector3 {
        let u = 0.5 * (u + 1.0);
        let v = 0.5 * (v + 1.0);
        self.sample_bilinear(u, v)
    }

    // https://en.wikipedia.org/wiki/Bilinear_interpolation
    pub fn sample_bilinear(&self, u: f64, v: f64) -> Vector3 {
        let x = u * self.image.width() as f64;
        let y = v * self.image.height() as f64;
        let x1 = x.floor();
        let y1 = y.floor();
        let x2 = x1 + 1.0;
        let y2 = y1 + 1.0;

        let p11 = self.sample_nearest_screen(x1 as u32, y1 as u32);
        let p12 = self.sample_nearest_screen(x1 as u32, y2 as u32);
        let p21 = self.sample_nearest_screen(x2 as u32, y1 as u32);
        let p22 = self.sample_nearest_screen(x2 as u32, y2 as u32);

        (
            p11 * (x2 - x) * (y2 - y) +
            p21 * (x - x1) * (y2 - y) +
            p12 * (x2 - x) * (y - y1) +
            p22 * (x - x1) * (y - y1)
        ) / ((x2- x1) * (y2 - y1))
    }

    pub fn sample_nearest(&self, u: f64, v: f64) -> Vector3 {
        let x = u * self.image.width() as f64;
        let y = v * self.image.height() as f64;
        self.sample_nearest_screen(x as u32, y as u32)
    }

    fn sample_nearest_screen(&self, x: u32, y: u32) -> Vector3 {
        let x = clamp(x,0, self.image.width() - 1);
        let y = clamp(y, 0, self.image.height() - 1);
        color::rgba_to_vector3(self.image.get_pixel(x, y))
    }
}
