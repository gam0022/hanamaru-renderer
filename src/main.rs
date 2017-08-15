extern crate num;
extern crate image;
extern crate time;

use std::fs::File;
use std::path::Path;

mod consts;
mod vector;
mod matrix;
mod scene;
mod camera;
mod renderer;
mod material;
mod brdf;
mod color;
mod texture;
mod math;
mod loader;
mod bvh;

use vector::Vector3;
use matrix::Matrix44;
use scene::{Scene, Sphere, AxisAlignedBoundingBox, Mesh, BvhMesh, Skybox};
use camera::{Camera, LensShape};
use material::{Material, SurfaceType};
use texture::Texture;
use renderer::{Renderer, DebugRenderer, PathTracingRenderer};
use color::Color;
use loader::ObjLoader;

fn render() {
    let width = 800;
    let height = 600;
    let mut imgbuf = image::ImageBuffer::new(width, height);

    let camera = Camera::new(
        Vector3::new(0.0, 3.0, 9.0),// eye
        Vector3::new(0.0, 1.0, 0.0),// target
        Vector3::new(0.0, 1.0, 0.0),// y_up
        20.0,// fov

        LensShape::Circle,// lens shape
        0.15,// aperture
        6.5// focus_distance
    );

    let scene = Scene {
        elements: vec![
//            Box::new(Sphere {
//                center: Vector3::new(0.0, 1.0, 0.0),
//                radius: 1.0,
//                material: Material {
//                    surface: SurfaceType::GGX { roughness: 0.2 },
//                    albedo: Texture::from_color(Color::new(1.0, 0.1, 0.1)),
//                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(3.0, 3.0, 1.1)),
//                }
//            }),
//            Box::new(Sphere {
//                center: Vector3::new(2.0, 0.5, -1.0),
//                radius: 0.5,
//                material: Material {
//                    surface: SurfaceType::Refraction { refractive_index: 1.5 },
//                    albedo: Texture::from_color(Color::new(0.5, 0.5, 1.0)),
//                    emission: Texture::black(),
//                }
//            }),
//            Box::new(Sphere {
//                center: Vector3::new(-3.0, 1.5, -1.0),
//                radius: 1.5,
//                material: Material {
//                    surface: SurfaceType::GGX { roughness: 0.0 },
//                    albedo: Texture::from_color(Color::new(1.0, 1.0, 1.0)),
//                    emission: Texture::black(),
//                }
//            }),
//            Box::new(Sphere {
//                center: Vector3::new(1.0, 0.8, 1.1),
//                radius: 0.8,
//                material: Material {
//                    surface: SurfaceType::Refraction { refractive_index: 1.2 },
//                    albedo: Texture::from_color(Color::new(0.7, 1.0, 0.7)),
//                    emission: Texture::black(),
//                }
//            }),
//            Box::new(Sphere {
//                center: Vector3::new(3.0, 1.0, 0.0),
//                radius: 1.0,
//                material: Material {
//                    surface: SurfaceType::GGXReflection { roughness: 0.2, refractive_index: 1.2 },
//                    albedo: Texture::from_color(Color::new(1.0, 0.5, 1.0)),
//                    emission: Texture::black(),
//                }
//            }),
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/bunny/bunny_face1000.obj",
                //"models/octahedron.obj",
                Matrix44::scale_linear(2.0) * Matrix44::translate(0.0, 0.0, 0.5) * Matrix44::rotate_y(-0.5),
                Material {
                    surface: SurfaceType::GGX { roughness: 0.1 },
                    albedo: Texture::from_color(Color::new(1.0, 0.2, 0.2)),
                    emission: Texture::black(),
                },
            ))),

            // 床
            Box::new(AxisAlignedBoundingBox {
                left_bottom: Vector3::new(-5.0, -1.0, -5.0),
                right_top: Vector3::new(5.0, 0.0, 5.0),
                material: Material {
                    surface: SurfaceType::Diffuse {},
                    albedo: Texture::from_path("textures/2d/checkered_512.jpg"),
                    emission: Texture::black(),
                }
            }),
            //Box::new(Plane{ center: Vector3::new(0.0, 0.0, 0.0), normal: Vector3::new(0.0, 1.0, 0.0), material: Material {
            //    surface: SurfaceType::Diffuse {},
            //    albedo: Texture::from_path("textures/2d/diamond_512.png"),
            //    emission: Texture::black(),
            //}}),
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

    //let mut renderer = DebugRenderer{};
    let mut renderer = PathTracingRenderer::new();
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
