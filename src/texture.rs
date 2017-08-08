extern crate image;

use image::{DynamicImage, GenericImage};
use std::path::Path;
use std::fmt;

use vector::{Vector3, Vector2};
use color::Color;
use color;

pub struct ImageTexture {
    pub image: DynamicImage,
}

fn clamp(x: u32, min: u32, max: u32) -> u32 {
    if x < min { min } else if x > max { max } else { x }
}

impl ImageTexture {
    pub fn new(path: &str) -> ImageTexture {
        ImageTexture {
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
        color::rgba_to_color(self.image.get_pixel(x, y))
    }
}

impl fmt::Debug for ImageTexture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Texture {{ width: {}, height: {} }}", self.image.width(), self.image.height())
    }
}

#[derive(Debug)]
pub struct Texture {
    pub image_texture: Option<ImageTexture>,
    pub color: Color,
}

impl Texture {
    pub fn new(path: &str, color: Color) -> Texture {
        Texture {
            image_texture: Some(ImageTexture::new(path)),
            color: color,
        }
    }

    pub fn from_path(path: &str) -> Texture {
        Texture {
            image_texture: Some(ImageTexture::new(path)),
            color: Vector3::one(),
        }
    }

    pub fn from_color(color: Color) -> Texture {
        Texture {
            image_texture: None,
            color: color,
        }
    }

    pub fn white() -> Texture {
        Texture::from_color(Color::one())
    }

    pub fn black() -> Texture {
        Texture::from_color(Color::zero())
    }

    pub fn sample(&self, uv: Vector2) -> Color {
        if let Some(ref tex) = self.image_texture {
            tex.sample_bilinear(uv.x, uv.y) * self.color
        } else {
            self.color
        }
    }
}
