extern crate num;
extern crate image;
extern crate time;

use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::fs;
use std::io::{BufWriter, Write};
use num::Float;

mod config;
mod vector;
mod matrix;
mod scene;
mod camera;
mod renderer;
mod material;
mod bsdf;
mod color;
mod texture;
mod math;
mod loader;
mod bvh;

use vector::Vector3;
use matrix::Matrix44;
use scene::{Scene, Sphere, AxisAlignedBoundingBox, BvhMesh, Skybox};
use camera::{Camera, LensShape};
use material::{Material, SurfaceType};
use texture::Texture;
use renderer::{Renderer, DebugRenderer, PathTracingRenderer};
use color::Color;
use loader::ObjLoader;

fn render() {
    let mut imgbuf = image::ImageBuffer::new(800, 600);
    //let mut imgbuf = image::ImageBuffer::new(1280, 720);
    //let mut imgbuf = image::ImageBuffer::new(1920, 1080);

    let camera = Camera::new(
        Vector3::new(0.0, 3.0, 9.0),// eye
        Vector3::new(0.0, 1.0, 0.0),// target
        Vector3::new(0.0, 1.0, 0.0),// y_up
        17.0,// fov

        LensShape::Circle,// lens shape
        0.15 * 0.0,// aperture
        6.5// focus_distance
    );

    let scene = Scene {
        elements: vec![
            // うさぎ
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/bunny/bunny_face1000.obj",
                Matrix44::scale_linear(1.0) * Matrix44::translate(1.3, 0.0, 0.5) * Matrix44::rotate_y(-0.5),
                Material {
                    surface: SurfaceType::GGX,
                    albedo: Texture::from_color(Color::new(1.0, 0.2, 0.2)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.1)),
                },
            ))),

            // Dia
            Box::new(ObjLoader::load(
                "models/dia/dia.obj",
                Matrix44::translate(-0.7, 0.0, 0.0) * Matrix44::scale_linear(2.0) * Matrix44::rotate_y(0.1) * Matrix44::rotate_x(-40.9771237.to_radians()),
                Material {
                    surface: SurfaceType::GGXReflection { refractive_index: 1.4 },
                    albedo: Texture::from_color(Color::new(1.0, 1.0, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.01)),
                },
            )),

            // 金属球
            /*Box::new(Sphere {
                center: Vector3::new(-3.0, 1.0, 0.0),
                radius: 1.0,
                material: Material {
                    surface: SurfaceType::GGX,
                    albedo: Texture::from_color(Color::new(0.1, 0.6, 0.9)),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(3.0, 3.0, 1.1)),
                    roughness: Texture::from_color(Color::from_one(0.2)),
                }
            }),

            // 磨りガラス
            Box::new(Sphere {
                center: Vector3::new(-2.0, 0.5, 2.0),
                radius: 0.5,
                material: Material {
                    surface: SurfaceType::GGXReflection { refractive_index: 1.2 },
                    albedo: Texture::from_color(Color::new(0.1, 1.0, 0.2)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.1)),
                }
            }),*/

            // 背後にある地図ガラス
            Box::new(AxisAlignedBoundingBox {
                left_bottom: Vector3::new(-4.0, 0.0, -3.3),
                right_top: Vector3::new(4.0, 3.0, -3.0),
                material: Material {
                    surface: SurfaceType::GGXReflection { refractive_index: 1.2 },
                    albedo: Texture::white(),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(3.0, 3.0, 1.1)),
                    roughness: Texture::black(),
                }
            }),

            // エリアライト
            Box::new(AxisAlignedBoundingBox {
                left_bottom: Vector3::new(-5.0, -5.0, 10.0),
                right_top: Vector3::new(5.0, 5.0, 10.3),
                material: Material {
                    surface: SurfaceType::GGXReflection { refractive_index: 1.2 },
                    albedo: Texture::white(),
                    emission: Texture::from_color(Color::from_one(2.0)),
                    roughness: Texture::black(),
                }
            }),

            // Icosphere
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/blender/icosphere_meshlab.obj",
                Matrix44::scale_linear(0.7) * Matrix44::translate(4.0, 1.0, 2.0),
                Material {
                    surface: SurfaceType::GGXReflection { refractive_index: 1.2 },
                    albedo: Texture::from_color(Color::new(0.2, 0.2, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.1)),
                },
            ))),

            // 床
            Box::new(AxisAlignedBoundingBox {
                left_bottom: Vector3::new(-5.0, -1.0, -5.0),
                right_top: Vector3::new(5.0, 0.0, 5.0),
                material: Material {
                    surface: SurfaceType::GGX,
                    //albedo:  Texture::white(),
                    albedo: Texture::from_path("textures/2d/stone03.jpg"),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.9)),
                }
            }),
        ],
        skybox: Skybox::new(
            "textures/cube/LancellottiChapel/posx.jpg",
            "textures/cube/LancellottiChapel/negx.jpg",
            "textures/cube/LancellottiChapel/posy.jpg",
            "textures/cube/LancellottiChapel/negy.jpg",
            "textures/cube/LancellottiChapel/posz.jpg",
            "textures/cube/LancellottiChapel/negz.jpg",
        ),
    };

    let mut renderer = DebugRenderer{};
    let mut renderer = PathTracingRenderer::new();
    renderer.render(&scene, &camera, &mut imgbuf);

    let ref mut fout = File::create(&Path::new("test.png")).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(fout, image::PNG);
}

fn main() {
    let begin = time::now();
    render();
    let end = time::now();
    let message = format!("total {} sec.", (end - begin).num_milliseconds() as f64 * 0.001);

    println!("{}", message);

    let mut f = BufWriter::new(fs::File::create("result.txt").unwrap());
    let _ = f.write_all(message.as_bytes());
}
