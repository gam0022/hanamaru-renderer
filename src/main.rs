extern crate num;
extern crate image;
use std::fs::File;
use std::path::Path;

mod consts;
mod vector;
mod scene;
mod renderer;
mod material;

use vector::Vector3;
use scene::{Scene, CameraBuilder, Sphere, Plane};
use material::{Material, SurfaceType};
use renderer::{Renderer, DebugRenderer};

#[allow(dead_code)]
fn main_gradation() {
    let width = 800;
    let height = 600;

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
    let mut imgbuf = image::ImageBuffer::new(width, height);

    let camera = CameraBuilder::new()
        .eye(Vector3::new(0.0, 3.0, 9.0))
        .target(Vector3::new(0.0, 1.0, 0.0))
        .y_up(Vector3::new(0.0, 1.0, 0.0))
        .zoom(3.0)
        .finalize();

    let scene = Scene {
        elements: vec![
            Box::new(Sphere{ center: Vector3::new(0.0, 1.0, 0.0), radius: 1.0, material: Material {
                albedo: Vector3::new(1.0, 0.5, 0.5),
                emission: Vector3::zero(),
                surface: SurfaceType::Diffuse {},
            }}),
            Box::new(Sphere{ center: Vector3::new(2.0, 0.5, -1.0), radius: 0.5, material: Material {
                albedo: Vector3::new(0.5, 0.5, 1.0),
                emission: Vector3::zero(),
                surface: SurfaceType::Diffuse {},
            }}),
            Box::new(Plane{ center: Vector3::new(0.0, 0.0, 0.0), normal: Vector3::new(0.0, 1.0, 0.0), material: Material {
                albedo: Vector3::new(1.0, 1.0, 1.0),
                emission: Vector3::zero(),
                surface: SurfaceType::Diffuse {},
            }}),
        ],
    };

    let renderer = DebugRenderer{};
    renderer.render(&scene, &camera, &mut imgbuf);

    let ref mut fout = File::create(&Path::new("test.png")).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(fout, image::PNG);
}
