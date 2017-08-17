extern crate image;
extern crate rand;
extern crate rayon;
extern crate time;

use std::fs::File;
use std::path::Path;
use std::process;
use time::Tm;
use image::{ImageBuffer, Rgb};
use self::rand::{thread_rng, Rng};
use self::rayon::prelude::*;

use consts;
use vector::{Vector3, Vector2};
use scene::Scene;
use camera::{Camera, Ray};
use material::SurfaceType;
use bsdf;
use color::{Color, color_to_rgb, linear_to_gamma};
use math::saturate;

pub trait Renderer: Sync {
    fn render_single_thread(&mut self, scene: &Scene, camera: &Camera, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let resolution = Vector2::new(imgbuf.width() as f64, imgbuf.height() as f64);
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let frag_coord = Vector2::new(x as f64, resolution.y - y as f64);
            let liner = self.supersampling(scene, camera, &frag_coord, &resolution);
            let gamma = linear_to_gamma(liner);
            *pixel = color_to_rgb(gamma);
        }
    }

    fn render(&mut self, scene: &Scene, camera: &Camera, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let resolution = Vector2::new(imgbuf.width() as f64, imgbuf.height() as f64);
        for y in 0..imgbuf.height() {
            let input: Vec<u32> = (0..imgbuf.width()).collect();
            let mut output = vec![];
            input.par_iter()
                .map(|&x| {
                    let frag_coord = Vector2::new(x as f64, resolution.y - y as f64);
                    let liner = self.supersampling(scene, camera, &frag_coord, &resolution);
                    let gamma = linear_to_gamma(liner);
                    color_to_rgb(gamma)
                }).collect_into(&mut output);
            for (x, pixel) in output.iter().enumerate() {
                imgbuf.put_pixel(x as u32, y, *pixel);
            }

            self.report_progress(y, resolution.y, imgbuf);
        }
    }

    fn report_progress(&mut self, y: u32, height: f64, imgbuf: &ImageBuffer<Rgb<u8>, Vec<u8>>);

    fn save_progress_image(path: &str, imgbuf: &ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let ref mut fout = File::create(&Path::new(path)).unwrap();
        let _ = image::ImageRgb8(imgbuf.clone()).save(fout, image::PNG);
    }

    fn supersampling(&self, scene: &Scene, camera: &Camera, frag_coord: &Vector2, resolution: &Vector2) -> Color {
        let mut accumulation = Color::zero();

        for sy in 0..consts::SUPERSAMPLING {
            for sx in 0..consts::SUPERSAMPLING {
                let offset = Vector2::new(sx as f64, sy as f64) / consts::SUPERSAMPLING as f64 - 0.5;
                let normalized_coord = ((*frag_coord + offset) * 2.0 - *resolution) / resolution.x.min(resolution.y);
                let color = self.calc_pixel(&scene, &camera, &normalized_coord);
                accumulation += color;
            }
        }

        accumulation / (consts::SUPERSAMPLING * consts::SUPERSAMPLING) as f64
    }

    fn calc_pixel(&self, scene: &Scene, camera: &Camera, normalized_coord: &Vector2) -> Color;
}

pub struct DebugRenderer;

impl Renderer for DebugRenderer {
    #[allow(unused_variables)]
    fn calc_pixel(&self, scene: &Scene, camera: &Camera, normalized_coord: &Vector2) -> Color {
        let mut ray = camera.ray(&normalized_coord);
        let light_direction = Vector3::new(1.0, 2.0, 1.0).normalize();

        let mut accumulation = Color::zero();
        let mut reflection = Color::one();

        for _ in 1..consts::DEBUG_BOUNCE_LIMIT {
            let (hit, intersection) = scene.intersect(&ray);

            let shadow_ray = Ray {
                origin: intersection.position + intersection.normal * consts::OFFSET,
                direction: light_direction,
            };
            let (shadow_hit, shadow_intersection) = scene.intersect(&shadow_ray);
            let shadow = if shadow_hit { 0.5 } else { 1.0 };

            if hit {
                match intersection.material.surface {
                    SurfaceType::Specular => {
                        ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                        ray.direction = ray.direction.reflect(&intersection.normal);
                        reflection *= intersection.material.albedo;
                    }
                    // 鏡面以外は拡散面として処理する
                    _ => {
                        let diffuse = intersection.normal.dot(&light_direction).max(0.0);
                        let color = intersection.material.emission + intersection.material.albedo * diffuse * shadow;
                        reflection *= color;
                        accumulation += reflection;
                        break;
                    }
                }
            } else {
                reflection = reflection * intersection.material.emission;
                accumulation += reflection;
                break;
            }
        }

        accumulation
    }

    #[allow(unused_variables)]
    fn report_progress(&mut self, y: u32, height: f64, imgbuf: &ImageBuffer<Rgb<u8>, Vec<u8>>) {
        // Nop
    }
}

pub struct PathTracingRenderer {
    begin: Tm,
    last_report_image: Tm,
    report_image_counter: u32,
}

impl Renderer for PathTracingRenderer {
    fn calc_pixel(&self, scene: &Scene, camera: &Camera, normalized_coord: &Vector2) -> Color {
        let mut rng = thread_rng();
        let mut all_accumulation = Vector3::zero();
        for _ in 1..consts::PATHTRACING_SAMPLING {
            let mut ray = camera.ray_with_dof(&normalized_coord, &mut rng);
            let mut accumulation = Color::zero();
            let mut reflection = Color::one();

            for _ in 1..consts::PATHTRACING_BOUNCE_LIMIT {
                let random = rng.gen::<(f64, f64)>();
                let (hit, mut intersection) = scene.intersect(&ray);

                if hit {
                    match intersection.material.surface {
                        SurfaceType::Diffuse => {
                            ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                            ray.direction = bsdf::importance_sample_diffuse(random, &intersection.normal);
                        }
                        SurfaceType::Specular => {
                            ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                            ray.direction = ray.direction.reflect(&intersection.normal);
                        }
                        SurfaceType::Refraction { refractive_index } => {
                            bsdf::sample_refraction(random, &intersection.normal, refractive_index, &intersection, &mut ray);
                        }
                        SurfaceType::GGX { roughness } => {
                            let alpha2 = bsdf::roughness_to_alpha2(roughness);
                            let half = bsdf::importance_sample_ggx(random, &intersection.normal, alpha2);
                            let next_direction = ray.direction.reflect(&half);

                            // 半球外が選ばれた場合はBRDFを0にする
                            // 真値よりも暗くなるので、サンプリングやり直す方が理想的ではありそう
                            if intersection.normal.dot(&next_direction).is_sign_negative() {
                                break;
                            } else {
                                let view = -ray.direction;
                                let v_dot_n = saturate(view.dot(&intersection.normal));
                                let l_dot_n = saturate(next_direction.dot(&intersection.normal));
                                let v_dot_h = saturate(view.dot(&half));
                                let h_dot_n = saturate(half.dot(&intersection.normal));

                                let g = bsdf::g_smith_joint(l_dot_n, v_dot_n, alpha2);
                                // albedoをフレネル反射率のパラメータのF0として扱う
                                let f = bsdf::f_schlick(v_dot_h, &intersection.material.albedo);
                                let weight = g * f * v_dot_h / (h_dot_n * v_dot_n);
                                intersection.material.albedo *= weight;
                            }

                            ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                            ray.direction = next_direction;
                        }
                        SurfaceType::GGXReflection { refractive_index, roughness } => {
                            let alpha2 = bsdf::roughness_to_alpha2(roughness);
                            let half = bsdf::importance_sample_ggx(random, &intersection.normal, alpha2);
                            bsdf::sample_refraction(random, &half, refractive_index, &intersection, &mut ray);
                        }
                    }
                }

                accumulation += reflection * intersection.material.emission;
                reflection *= intersection.material.albedo;

                if !hit { break; }
            }
            all_accumulation += accumulation;
        }

        all_accumulation / consts::PATHTRACING_SAMPLING as f64
    }

    fn report_progress(&mut self, y: u32, height: f64, imgbuf: &ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let progress = (y as f64 + 1.0) / height * 100.0;

        let now = time::now();
        let passed_time = (now - self.begin).num_milliseconds() as f64 * 0.001;

        println!("rendering: {:.2} % {:.3} sec.", progress, passed_time);

        let interval_time = (now - self.last_report_image).num_milliseconds() as f64 * 0.001;
        if interval_time >= consts::REPORT_INTERVAL_SEC {
            // save progress image
            let path = format!("progress_{:>03}.png", self.report_image_counter);
            println!("output progress image: {}", path);
            PathTracingRenderer::save_progress_image(&path, imgbuf);
            self.report_image_counter += 1;
            self.last_report_image = now;
        }

        if passed_time > consts::TIME_LIMIT_SEC {
            // die when time limit exceeded
            let path = "result_tle.png";
            println!("time limit exceeded: {:.3} sec. {}", passed_time, path);
            PathTracingRenderer::save_progress_image(path, imgbuf);
            process::exit(1);
        }
    }
}

impl PathTracingRenderer {
    pub fn new() -> PathTracingRenderer {
        let now = time::now();
        PathTracingRenderer {
            begin: now,
            last_report_image: now,
            report_image_counter: 0,
        }
    }
}
