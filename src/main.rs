extern crate num;
extern crate image;
use std::fs::File;
use std::path::Path;

mod vector;
mod consts;
mod scene;
mod renderer;

use vector::{Vector3, Vector2};
use scene::{Camera, Ray, Intersectable, Sphere, Intersection};
use renderer::DebugRenderer;

fn main_test() {
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

fn main() {
    let width = 800;
    let height = 600;
    let resolution = Vector2::new(width as f64, height as f64);

    let mut imgbuf = image::ImageBuffer::new(width, height);

    let camera = Camera::new(
        Vector3::new(0.0, 0.0, -3.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        1.5
    );
    println!("{:?}", camera);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let frag_coord = Vector2::new(x as f64, y as f64);
        let uv = (frag_coord * 2.0 - resolution) / resolution.x.min(resolution.y);
        let color = DebugRenderer::test(&camera, &uv);
        *pixel = image::Rgb([
            (255.0 * color.x) as u8,
            (255.0 * color.y) as u8,
            (255.0 * color.z) as u8,
        ]);
    }

    let ref mut fout = File::create(&Path::new("test.png")).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(fout, image::PNG);
}
