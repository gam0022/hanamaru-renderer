extern crate num;
extern crate image;
extern crate time;
use std::fs::File;
use std::path::Path;

mod consts;
mod vector;
mod scene;
mod renderer;
mod material;
mod brdf;
mod random;
mod color;

use vector::Vector3;
use scene::{Scene, CameraBuilder, Sphere, Plane, Skybox};
use material::{Material, SurfaceType};
use renderer::{Renderer, DebugRenderer, PathTracingRenderer};

fn render() {
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
                emission: Vector3::from_one(0.0),
                surface: SurfaceType::Diffuse {},
            }}),
            Box::new(Sphere{ center: Vector3::new(-3.0, 1.5, -1.0), radius: 1.5, material: Material {
                albedo: Vector3::new(1.0, 1.0, 1.0),
                emission: Vector3::zero(),
                surface: SurfaceType::Specular {},
            }}),
            Box::new(Plane{ center: Vector3::new(0.0, 0.0, 0.0), normal: Vector3::new(0.0, 1.0, 0.0), material: Material {
                albedo: Vector3::new(0.8, 0.8, 0.8),
                emission: Vector3::zero(),
                surface: SurfaceType::Diffuse {},
            }}),
        ],
        skybox: Skybox::new(
            "textures/cube/pisa/px.png",
            "textures/cube/pisa/nx.png",
            "textures/cube/pisa/py.png",
            "textures/cube/pisa/ny.png",
            "textures/cube/pisa/pz.png",
            "textures/cube/pisa/nz.png",
        ),
    };

    //let renderer = DebugRenderer{};
    let renderer = PathTracingRenderer{};
    renderer.render(&scene, &camera, &mut imgbuf);

    let ref mut fout = File::create(&Path::new("test.png")).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(fout, image::PNG);
}

fn main() {
    let begin = time::now();
    render();
    let end = time::now();
    println!("total {} sec.", (end - begin).num_milliseconds() as f64 * 0.001);
}