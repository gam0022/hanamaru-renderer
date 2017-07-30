extern crate num;
extern crate image;
use std::fs::File;
use std::path::Path;

mod vector;
mod consts;
mod scene;

use vector::{Vector3, Vector2};
use scene::{Ray, Sphere};

fn main() {
    let width = 800;
    let height = 600;

    scene::test();

    let mut imgbuf = image::ImageBuffer::new(width, height);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let u = x as f64 / width as f64;
        let v = y as f64 / height as f64;
        *pixel = image::Rgb([(255.0 * u) as u8, (255.0 * v) as u8, 127]);
    }

    let ref mut fout = File::create(&Path::new("test.png")).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(fout, image::PNG);
}
