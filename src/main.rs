extern crate num;
extern crate image;
extern crate time;
extern crate rand;

use std::fs::File;
use std::path::Path;
use std::fs;
use std::io::{BufWriter, Write};
use num::Float;
use self::rand::{Rng, SeedableRng, StdRng};

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
use scene::{Scene, BvhScene, Sphere, Cuboid, BvhMesh, Skybox};
use bvh::Aabb;
use camera::{Camera, LensShape};
use material::{Material, SurfaceType};
use texture::Texture;
use renderer::{Renderer, DebugRenderer, DebugRenderMode, PathTracingRenderer};
use color::{Color, hsv_to_rgb};
use loader::ObjLoader;

fn tee(f: &mut BufWriter<File>, message: &String) {
    println!("{}", message);
    let _ = f.write_all(message.as_bytes());
    let _ = f.write(b"\n");
}

fn init_scene() -> (Camera, Scene) {
    let seed: &[_] = &[870, 2000, 304, 3];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    let camera = Camera::new(
        Vector3::new(0.0, 2.5, 9.0),// eye
        Vector3::new(0.0, 1.0, 0.0),// target
        Vector3::new(0.0, 1.0, 0.0).normalize(),// y_up
        17.0,// fov

        LensShape::Circle,// lens shape
        0.15,// * 0.0,// aperture
        8.5// focus_distance
    );

    let mut scene = Scene {
        elements: vec![
            // うさぎ右
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/bunny/bunny_face1000.obj",
                Matrix44::scale_linear(1.5) * Matrix44::translate(1.2, 0.0, 0.0) * Matrix44::rotate_y(0.2),
                Material {
                    surface: SurfaceType::Refraction { refractive_index: 1.5 },
                    albedo: Texture::from_color(Color::new(0.7, 0.7, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.1)),
                },
            ))),

            // うさぎ左
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/bunny/bunny_face1000_flip.obj",
                Matrix44::scale(1.5, 1.5, 1.5) * Matrix44::translate(-1.2, 0.0, 0.0) * Matrix44::rotate_y(-0.2),

                Material {
                    surface: SurfaceType::GGX,
                    albedo: Texture::from_color(Color::new(1.0, 0.2, 0.2)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.1)),
                },
            ))),

            // 背後にある地図ガラス
            /*Box::new(Cuboid {
                aabb: Aabb {
                    min: Vector3::new(-4.0, 0.0, -3.6),
                    max: Vector3::new(4.0, 3.0, -3.5),
                },
                material: Material {
                    surface: SurfaceType::GGXReflection { refractive_index: 1.2 },
                    albedo: Texture::white(),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(3.0, 3.0, 1.1)),
                    roughness: Texture::from_color(Color::from_one(0.3)),
                }
            }),*/

            // 固定のダイヤモンド
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/dia/dia.obj",
                Matrix44::translate(3.1, 0.0, 0.8) * Matrix44::scale_linear(1.0) * Matrix44::rotate_y(-0.5) * Matrix44::rotate_x(40.35.to_radians()),
                Material {
                    surface: SurfaceType::GGXReflection { refractive_index: 2.42 },
                    albedo: Texture::from_color(Color::new(1.0, 1.0, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.01)),
                },
            ))),

            // 地球のテクスチャを光源にした球体
            Box::new(Sphere {
                center: Vector3::new(0.0, 0.5, -0.5),
                radius: 0.5,
                material: Material {
                    surface: SurfaceType::GGX,
                    albedo: Texture::white(),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(5.0, 5.0, 2.0)),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),

            // 地球のテクスチャをラフネスにした球体
            Box::new(Sphere {
                center: Vector3::new(-3.5, 0.5, 0.0),
                radius: 0.5,
                material: Material {
                    surface: SurfaceType::GGX,
                    albedo: Texture::from_color(Color::new(1.0, 1.0, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_path("textures/2d/earth_inverse_2048.jpg"),
                },
            }),

            // 床
            Box::new(Cuboid {
                aabb: Aabb {
                    min: Vector3::new(-5.0, -1.0, -5.0),
                    max: Vector3::new(5.0, 0.0, 5.0),
                },
                material: Material {
                    surface: SurfaceType::Diffuse,
                    //albedo:  Texture::white(),
                    albedo: Texture::from_path("textures/2d/stone03.jpg"),
                    //albedo: Texture::from_path("textures/2d/checkered_v2_512.png"),
                    emission: Texture::black(),
                    roughness: Texture::black(),
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

    // 金属の球体
    let mut count = 0;
    while count < 8 {
        let px = rng.gen_range(-2.5, 3.5);
        let py = 0.0;//rng.gen_range(0.0, 3.0);
        let pz = rng.gen_range(-2.0, 3.0);
        let r = rng.gen_range(0.2, 0.4);

        if scene.add_with_check_collisions((Box::new(Sphere {
            center: Vector3::new(px, r + py, pz),
            radius: r,
            material: Material {
                surface: SurfaceType::GGX,
                albedo: Texture::from_color(hsv_to_rgb(Color::new(0.2 + 0.1 * count as f64, 1.0, 1.0))),
                emission: Texture::black(),
                roughness: Texture::from_color(Color::from_one(rng.gen_range(0.0, 0.2))),
            },
        }))) {
            count += 1;
        }
    }

    // 床に落ちているダイヤモンド
    count = 0;
    while count < 12 {
        let px = rng.gen_range(-4.5, 4.5);
        let py = 0.0;
        let pz = rng.gen_range(-4.5, 4.5);
        let s = rng.gen_range(0.5, 1.5);
        let ry = rng.gen_range(-180.0.to_radians(), 180.0.to_radians());

        if scene.add_with_check_collisions(Box::new(BvhMesh::from_mesh(ObjLoader::load(
            "models/dia/dia.obj",
            Matrix44::translate(px, py, pz) * Matrix44::scale_linear(s) * Matrix44::rotate_y(ry) * Matrix44::rotate_x(40.35.to_radians()),
            Material {
                surface: SurfaceType::GGXReflection { refractive_index: 2.42 },
                albedo: Texture::from_color(Color::new(1.0, 1.0, 1.0)),
                emission: Texture::black(),
                roughness: Texture::from_color(Color::from_one(0.01)),
            },
        )))) {
            count += 1;
        }
    }

    // 空中浮遊しているダイヤモンド
    count = 0;
    while count < 30 {
        let px = rng.gen_range(-4.5, 4.5);
        let py = rng.gen_range(0.0, 4.0);
        let pz = rng.gen_range(-4.5, 4.5);
        let s = rng.gen_range(0.2, 1.0);
        let ry = rng.gen_range(-180.0.to_radians(), 180.0.to_radians());
        let rx = rng.gen_range(-180.0.to_radians(), 180.0.to_radians());

        if scene.add_with_check_collisions(Box::new(BvhMesh::from_mesh(ObjLoader::load(
            "models/dia/dia.obj",
            Matrix44::translate(px, py, pz) * Matrix44::scale_linear(s) * Matrix44::rotate_y(ry) * Matrix44::rotate_x(rx),
            Material {
                surface: SurfaceType::GGXReflection { refractive_index: 1.4 },
                albedo: Texture::from_color(Color::new(1.0, 1.0, 1.0)),
                emission: Texture::black(),
                roughness: Texture::from_color(Color::from_one(0.01)),
            },
        )))) {
            count += 1;
        }
    }

    (camera, scene)
}

fn render(width: u32, height: u32, sampling: u32) {
    let mut imgbuf = image::ImageBuffer::new(width, height);
    let mut renderer = DebugRenderer{ mode: DebugRenderMode::DepthFromFocus };
    let mut renderer = PathTracingRenderer::new(sampling);

    let begin = time::now();
    let (camera, scene) = init_scene();
    let end = time::now();
    let total_sec = (end - begin).num_milliseconds() as f64 * 0.001;
    println!("init scene: {} sec.", total_sec);

    renderer.render(&BvhScene::from_scene(scene), &camera, &mut imgbuf);

    let ref mut fout = File::create(&Path::new("test.png")).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(fout, image::PNG);
}


fn main() {
    let mut f = BufWriter::new(fs::File::create("result.txt").unwrap());

    let (width, height, sampling) = (800, 600, 5);// SVGA 480,000 pixel
    //let (width, height, sampling) = (1280, 960, 75);// QVGA 1,228,800 pixel
    //let (width, height, sampling) = (1920, 1080, 3);// FHD 2,073,600 pixel

    tee(&mut f, &format!("resolution: {}x{}.", width, height));
    tee(&mut f, &format!("sampling: {}x{} spp.", sampling, config::SUPERSAMPLING * config::SUPERSAMPLING));

    let begin = time::now();
    render(width, height, sampling);
    let end = time::now();

    let total_sec = (end - begin).num_milliseconds() as f64 * 0.001;
    let used_percent = total_sec / config::TIME_LIMIT_SEC as f64 * 100.0;
    let progress_per_used = 100.0 / used_percent;
    tee(&mut f, &format!("total {} sec. used {:.2} % (x {:.2})", total_sec, used_percent, progress_per_used));
}
