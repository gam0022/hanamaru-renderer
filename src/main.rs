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
mod texture;
mod math;

use vector::Vector3;
use scene::{Scene, CameraBuilder, Sphere, Plane, Skybox};
use material::{Material, SurfaceType};
use texture::Texture;
use renderer::{Renderer, DebugRenderer, PathTracingRenderer};
use color::Color;

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
                surface: SurfaceType::GGX { roughness: 0.2 },
                albedo: Texture::from_color(Color::new(1.0, 0.1, 0.1)),
                emission: Texture::black(),
            }}),
            Box::new(Sphere{ center: Vector3::new(2.0, 0.5, -1.0), radius: 0.5, material: Material {
                surface: SurfaceType::Refraction { refractive_index: 1.5 },
                albedo: Texture::from_color(Color::new(0.5, 0.5, 1.0)),
                emission: Texture::black(),
            }}),
            Box::new(Sphere{ center: Vector3::new(-3.0, 1.5, -1.0), radius: 1.5, material: Material {
                surface: SurfaceType::GGX { roughness: 0.0 },
                albedo: Texture::from_color(Color::new(1.0, 1.0, 1.0)),
                emission: Texture::black(),
            }}),
            Box::new(Sphere{ center: Vector3::new(1.0, 0.8, 1.1), radius: 0.8, material: Material {
                surface: SurfaceType::Refraction { refractive_index: 1.2 },
                albedo: Texture::from_color(Color::new(0.7, 1.0, 0.7)),
                emission: Texture::black(),
            }}),
            Box::new(Sphere{ center: Vector3::new(3.0, 1.0, 0.0), radius: 1.0, material: Material {
                surface: SurfaceType::GGXReflection { roughness: 0.2, refractive_index: 1.2 },
                albedo: Texture::from_color(Color::new(1.0, 0.5, 1.0)),
                emission: Texture::black(),
            }}),
            Box::new(Plane{ center: Vector3::new(0.0, 0.0, 0.0), normal: Vector3::new(0.0, 1.0, 0.0), material: Material {
                surface: SurfaceType::Diffuse {},
                albedo: Texture::from_path("textures/2d/diamond_512.png"),
                emission: Texture::black(),
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
