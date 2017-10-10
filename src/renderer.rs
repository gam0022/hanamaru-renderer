extern crate image;
extern crate rand;
extern crate rayon;
extern crate time;

use std::fs::File;
use std::path::Path;
use time::Tm;
use image::{ImageBuffer, Rgb};
use self::rand::{Rng, SeedableRng, StdRng};
use self::rayon::prelude::*;

use config;
use vector::{Vector3, Vector2};
use scene::SceneTrait;
use camera::{Camera, Ray};
use material::SurfaceType;
use bsdf;
use color::{Color, color_to_rgb, linear_to_gamma};
use math::saturate;

pub trait Renderer: Sync {
    fn max_sampling(&self) -> u32;

    fn calc_pixel(&self, scene: &SceneTrait, camera: &Camera, normalized_coord: &Vector2, sampling: u32) -> Color;

    fn render(&mut self, scene: &SceneTrait, camera: &Camera, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> u32 {
        let resolution = Vector2::new(imgbuf.width() as f64, imgbuf.height() as f64);
        let num_of_pixel = imgbuf.width() * imgbuf.height();
        let mut accumulation_buf = vec![Vector3::zero(); num_of_pixel as usize];

        // NOTICE: sampling is 1 origin
        for sampling in 1..(self.max_sampling() + 1) {
            accumulation_buf.par_iter_mut().enumerate().for_each(|(i, pixel)| {
                let y = i as u32 / imgbuf.width();
                let x = i as u32 - y * imgbuf.width();
                let frag_coord = Vector2::new(x as f64, (imgbuf.height() - y) as f64);
                *pixel += self.supersampling(scene, camera, &frag_coord, &resolution, sampling);
            });

            if self.report_progress(&accumulation_buf, sampling, imgbuf) {
                return sampling;
            }
        }

        self.max_sampling()
    }

    fn supersampling(&self, scene: &SceneTrait, camera: &Camera, frag_coord: &Vector2, resolution: &Vector2, sampling: u32) -> Color {
        let mut accumulation = Color::zero();

        for sy in 0..config::SUPERSAMPLING {
            for sx in 0..config::SUPERSAMPLING {
                let offset = Vector2::new(sx as f64, sy as f64) / config::SUPERSAMPLING as f64 - 0.5;
                let normalized_coord = ((*frag_coord + offset) * 2.0 - *resolution) / resolution.x.min(resolution.y);
                accumulation += self.calc_pixel(scene, camera, &normalized_coord, sampling);
            }
        }

        accumulation
    }

    fn report_progress(&mut self, accumulation_buf: &Vec<Vector3>, sampling: u32, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> bool;

    fn update_imgbuf(accumulation_buf: &Vec<Vector3>, sampling: u32, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let mut tmp: Vec<Rgb<u8>> = Vec::with_capacity((imgbuf.width() * imgbuf.height()) as usize);
        let scale = ((sampling * config::SUPERSAMPLING * config::SUPERSAMPLING) as f64).recip();
        accumulation_buf.par_iter().map(|p| {
            let liner = *p * scale;
            let gamma = linear_to_gamma(liner);
            color_to_rgb(gamma)
        }).collect_into(&mut tmp);

        for (i, pixel) in imgbuf.pixels_mut().enumerate() {
            *pixel = tmp[i];
        }
    }

    fn save_progress_image(path: &str, accumulation_buf: &Vec<Vector3>, sampling: u32, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let begin = time::now();
        Self::update_imgbuf(accumulation_buf, sampling, imgbuf);
        let end = time::now();
        println!("update_imgbuf: {:.3} sec", (end - begin).num_milliseconds() as f64 * 0.001);

        let ref mut fout = File::create(&Path::new(path)).unwrap();
        let _ = image::ImageRgb8(imgbuf.clone()).save(fout, image::PNG);
    }
}

pub enum DebugRenderMode {
    Shading,
    Normal,
    Depth,
    FocalPlane,
}

pub struct DebugRenderer {
    pub mode: DebugRenderMode,
}

impl Renderer for DebugRenderer {
    fn max_sampling(&self) -> u32 { 1 }

    fn calc_pixel(&self, scene: &SceneTrait, camera: &Camera, normalized_coord: &Vector2, _: u32) -> Color {
        let ray = camera.ray(&normalized_coord);
        let light_direction = Vector3::new(1.0, 2.0, -1.0).normalize();
        let (hit, intersection) = scene.intersect(&ray);
        if hit {
            match self.mode {
                DebugRenderMode::Shading => {
                    let shadow_ray = Ray {
                        origin: intersection.position + intersection.normal * config::OFFSET,
                        direction: light_direction,
                    };
                    let (shadow_hit, _) = scene.intersect(&shadow_ray);
                    let shadow = if shadow_hit { 0.5 } else { 1.0 };
                    let diffuse = intersection.normal.dot(&light_direction).max(0.0);
                    intersection.material.emission + intersection.material.albedo * diffuse * shadow
                }
                DebugRenderMode::Normal => intersection.normal,
                DebugRenderMode::Depth => Color::from_one(0.5 * intersection.distance / camera.focus_distance),
                DebugRenderMode::FocalPlane => Color::from_one((intersection.distance - camera.focus_distance).abs()),
            }
        } else {
            intersection.material.emission
        }
    }

    fn report_progress(&mut self, accumulation_buf: &Vec<Vector3>, sampling: u32, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> bool {
        // on finish
        Self::update_imgbuf(accumulation_buf, sampling, imgbuf);
        true
    }
}

pub struct PathTracingRenderer {
    sampling: u32,

    // for report_progress
    begin: Tm,
    last_report_progress: Tm,
    last_report_image: Tm,
    report_image_counter: u32,
}

impl Renderer for PathTracingRenderer {
    fn max_sampling(&self) -> u32 { self.sampling }

    fn calc_pixel(&self, scene: &SceneTrait, camera: &Camera, normalized_coord: &Vector2, sampling: u32) -> Color {
        // random generator
        let s = ((4.0 + normalized_coord.x) * 100870.0) as usize;
        let t = ((4.0 + normalized_coord.y) * 100304.0) as usize;
        let seed: &[_] = &[8700304, sampling as usize, s, t];
        let mut rng: StdRng = SeedableRng::from_seed(seed);// self::rand::thread_rng();
        let mut ray = camera.ray_with_dof(&normalized_coord, &mut rng);

        let mut accumulation = Color::zero();
        let mut reflection = Color::one();

        for _ in 1..config::PATHTRACING_BOUNCE_LIMIT {
            let random = rng.gen::<(f64, f64)>();
            let (hit, mut intersection) = scene.intersect(&ray);

            if hit {
                match intersection.material.surface {
                    SurfaceType::Diffuse => {
                        ray.origin = intersection.position + intersection.normal * config::OFFSET;
                        ray.direction = bsdf::importance_sample_diffuse(random, &intersection.normal);
                    }
                    SurfaceType::Specular => {
                        ray.origin = intersection.position + intersection.normal * config::OFFSET;
                        ray.direction = ray.direction.reflect(&intersection.normal);
                    }
                    SurfaceType::Refraction { refractive_index } => {
                        bsdf::sample_refraction(random, &intersection.normal.clone(), refractive_index, &mut intersection, &mut ray);
                    }
                    SurfaceType::GGX => {
                        let alpha2 = bsdf::roughness_to_alpha2(intersection.material.roughness);
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

                            // Masking-shadowing関数
                            let g = bsdf::g_smith_joint(l_dot_n, v_dot_n, alpha2);

                            // albedoをフレネル反射率のパラメータのF0として扱う
                            let f = bsdf::f_schlick(v_dot_h, &intersection.material.albedo);

                            let weight = f * saturate(g * v_dot_h / (h_dot_n * v_dot_n));
                            intersection.material.albedo *= weight;
                        }

                        ray.origin = intersection.position + intersection.normal * config::OFFSET;
                        ray.direction = next_direction;
                    }
                    SurfaceType::GGXRefraction { refractive_index } => {
                        let alpha2 = bsdf::roughness_to_alpha2(intersection.material.roughness);
                        let half = bsdf::importance_sample_ggx(random, &intersection.normal, alpha2);
                        bsdf::sample_refraction(random, &half, refractive_index, &mut intersection, &mut ray);
                    }
                }
            }

            accumulation += reflection * intersection.material.emission;
            reflection *= intersection.material.albedo;

            if !hit { break; }
        }

        accumulation
    }

    fn report_progress(&mut self, accumulation_buf: &Vec<Vector3>, sampling: u32, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> bool {
        let now = time::now();
        let used = (now - self.begin).num_milliseconds() as f64 * 0.001;
        let used_percent = used / config::TIME_LIMIT_SEC as f64 * 100.0;
        let from_last_sampling_sec = (now - self.last_report_progress).num_milliseconds() as f64 * 0.001;

        println!("rendering: {}x{} sampled (last {:.3} sec). total: {:.3} sec ({:.2} %).",
                 sampling, config::SUPERSAMPLING * config::SUPERSAMPLING,
                 from_last_sampling_sec,
                 used, used_percent);

        // reached time limit
        // 前フレームの所要時間から次のフレームが制限時間内に終るかを予測する。時間超過を防ぐために1.1倍に見積もる
        let offset = from_last_sampling_sec * 1.1;
        if used + offset > config::TIME_LIMIT_SEC {
            let path = format!("progress_{:>03}.png", self.report_image_counter);
            println!("reached time limit");
            println!("output final image: {}", path);
            println!("remain: {:.3} sec.", config::TIME_LIMIT_SEC - used);
            Self::save_progress_image(&path, accumulation_buf, sampling, imgbuf);
            return true;
        }

        // reached max sampling
        if sampling >= self.max_sampling() {
            let path = format!("progress_{:>03}.png", self.report_image_counter);
            println!("reached max sampling");
            println!("output final image: {}", path);
            println!("remain: {:.3} sec.", config::TIME_LIMIT_SEC - used);
            Self::save_progress_image(&path, accumulation_buf, sampling, imgbuf);
            return true;
        }

        // on interval time passed
        let from_last_report_image_sec = (now - self.last_report_image).num_milliseconds() as f64 * 0.001;
        if from_last_report_image_sec >= config::REPORT_INTERVAL_SEC {
            // save progress image
            let path = format!("progress_{:>03}.png", self.report_image_counter);
            println!("output progress image: {}", path);
            Self::save_progress_image(&path, accumulation_buf, sampling, imgbuf);
            self.report_image_counter += 1;
            self.last_report_image = now;
        }

        self.last_report_progress = now;
        false
    }
}

impl PathTracingRenderer {
    pub fn new(sampling: u32) -> PathTracingRenderer {
        let now = time::now();
        PathTracingRenderer {
            sampling: sampling,

            begin: now,
            last_report_progress: now,
            last_report_image: now,
            report_image_counter: 0,
        }
    }
}
