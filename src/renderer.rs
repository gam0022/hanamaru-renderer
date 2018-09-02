extern crate image;
extern crate rand;
extern crate rayon;
extern crate time;

use time::Tm;
use image::{ImageBuffer, Rgb};
use self::rand::{Rng, SeedableRng, StdRng};
use self::rayon::prelude::*;

use config;
use vector::{Vector3, Vector2};
use scene::{SceneTrait, Intersectable};
use camera::{Camera, Ray};
use color::{Color, color_to_rgb, linear_to_gamma};
use material::PointMaterial;
use tonemap;
use filter;

pub trait Renderer: Sync {
    fn max_sampling(&self) -> u32;

    fn calc_pixel(&self, scene: &SceneTrait, camera: &Camera, emissions: &Vec<&Box<Intersectable>>, normalized_coord: &Vector2, sampling: u32) -> Color;

    fn render(&mut self, scene: &SceneTrait, camera: &Camera, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> u32 {
        let resolution = Vector2::new(imgbuf.width() as f64, imgbuf.height() as f64);
        let num_of_pixel = imgbuf.width() * imgbuf.height();
        let mut accumulation_buf = vec![Vector3::zero(); num_of_pixel as usize];
        let emissions = scene.emissions();

        // NOTICE: sampling is 1 origin
        for sampling in 1..(self.max_sampling() + 1) {
            accumulation_buf.par_iter_mut().enumerate().for_each(|(i, pixel)| {
                let y = i as u32 / imgbuf.width();
                let x = i as u32 - y * imgbuf.width();
                let frag_coord = Vector2::new(x as f64, (imgbuf.height() - y) as f64);
                *pixel += self.supersampling(scene, camera, &emissions, &frag_coord, &resolution, sampling);
            });

            if self.report_progress(&accumulation_buf, sampling, imgbuf) {
                return sampling;
            }
        }

        self.max_sampling()
    }

    fn supersampling(&self, scene: &SceneTrait, camera: &Camera, emissions: &Vec<&Box<Intersectable>>, frag_coord: &Vector2, resolution: &Vector2, sampling: u32) -> Color {
        let mut accumulation = Color::zero();

        for sy in 0..config::SUPERSAMPLING {
            for sx in 0..config::SUPERSAMPLING {
                let offset = Vector2::new(sx as f64, sy as f64) / config::SUPERSAMPLING as f64 - 0.5;
                let normalized_coord = ((*frag_coord + offset) * 2.0 - *resolution) / resolution.x.min(resolution.y);
                accumulation += self.calc_pixel(scene, camera, &emissions, &normalized_coord, sampling);
            }
        }

        accumulation
    }

    fn report_progress(&mut self, accumulation_buf: &Vec<Vector3>, sampling: u32, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> bool;

    fn update_imgbuf(accumulation_buf: &Vec<Vector3>, sampling: u32, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let scale = ((sampling * config::SUPERSAMPLING * config::SUPERSAMPLING) as f64).recip();
        let width = imgbuf.width();
        let height = imgbuf.height();

        let mut tmp: Vec<_> = accumulation_buf.par_iter().map(|pixel| {
            let hdr = *pixel * scale;
            let ldr = tonemap::execute(&hdr);
            let gamma = linear_to_gamma(ldr);
            gamma
        }).collect();

        for _ in 0..config::BILATERAL_FILTER_ITERATION {
            tmp = tmp.par_iter().enumerate().map(|i_p| {
                let (index, pixel) = i_p;
                filter::execute(&pixel, index, &tmp, width, height)
            }).collect();
        }

        let rgbs: Vec<_> = tmp.par_iter().map(|pixel| {
            color_to_rgb(*pixel)
        }).collect();

        for (i, pixel) in imgbuf.pixels_mut().enumerate() {
            *pixel = rgbs[i];
        }
    }

    fn save_progress_image(path: &str, accumulation_buf: &Vec<Vector3>, sampling: u32, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let begin = time::now();
        Self::update_imgbuf(accumulation_buf, sampling, imgbuf);
        let end = time::now();
        println!("update_imgbuf: {:.3} sec", (end - begin).num_milliseconds() as f64 * 0.001);
        let _ = image::ImageRgb8(imgbuf.clone()).save(path);
    }
}

#[allow(dead_code)]
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

    fn calc_pixel(&self, scene: &SceneTrait, camera: &Camera, _emissions: &Vec<&Box<Intersectable>>, normalized_coord: &Vector2, _: u32) -> Color {
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
    time_limit_sec: f64,
    report_interval_sec: f64,

    // for report_progress
    begin: Tm,
    last_report_progress: Tm,
    last_report_image: Tm,
    report_image_counter: u32,
}

impl Renderer for PathTracingRenderer {
    fn max_sampling(&self) -> u32 { self.sampling }

    fn calc_pixel(&self, scene: &SceneTrait, camera: &Camera, emissions: &Vec<&Box<Intersectable>>, normalized_coord: &Vector2, sampling: u32) -> Color {
        // random generator
        let s = ((4.0 + normalized_coord.x) * 100870.0) as usize;
        let t = ((4.0 + normalized_coord.y) * 100304.0) as usize;
        let seed: &[_] = &[8700304, sampling as usize, s, t];
        let mut rng: StdRng = SeedableRng::from_seed(seed);// self::rand::thread_rng();
        let mut ray = camera.ray_with_dof(&normalized_coord, &mut rng);

        let mut accumulation = Color::zero();
        let mut reflectance = Color::one();

        for _ in 1..config::PATHTRACING_BOUNCE_LIMIT {
            let random = rng.gen::<(f64, f64)>();
            let (hit, mut intersection) = scene.intersect(&ray);
            let mut current_reflectance = 1.0;
            let mut bsdf_mis_weight_sum = 1.0;

            if hit {
                let view = &-ray.direction;
                if let Some(result) = intersection.material.sample(random, &intersection.position, view, &intersection.normal) {
                    if intersection.material.nee_available() && intersection.material.emission == Vector3::zero() {
                        let (nee_contribution, weight_sum) = PathTracingRenderer::next_event_estimation(
                            random, &result.ray.origin, view, &intersection.normal,
                            scene, &emissions, &intersection.material);
                        accumulation += reflectance * nee_contribution;
                        bsdf_mis_weight_sum = weight_sum;
                    }

                    ray = result.ray;
                    current_reflectance = result.reflectance;
                } else {
                    // 半球外をサンプリングしたら計算を打ち切る
                    break;
                }
            }

            accumulation += reflectance * bsdf_mis_weight_sum * intersection.material.emission;
            reflectance *= intersection.material.albedo * current_reflectance;

            if !hit || reflectance == Vector3::zero() { break; }
        }

        accumulation
    }

    fn report_progress(&mut self, accumulation_buf: &Vec<Vector3>, sampling: u32, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> bool {
        let now = time::now();
        let used = (now - self.begin).num_milliseconds() as f64 * 0.001;
        let used_percent = used / self.time_limit_sec as f64 * 100.0;
        let from_last_sampling_sec = (now - self.last_report_progress).num_milliseconds() as f64 * 0.001;

        println!("rendering: {}x{} sampled (last {:.3} sec). total: {:.3} sec ({:.2} %).",
                 sampling, config::SUPERSAMPLING * config::SUPERSAMPLING,
                 from_last_sampling_sec,
                 used, used_percent);

        // reached time limit
        // 前フレームの所要時間から次のフレームが制限時間内に終るかを予測する。時間超過を防ぐために1.1倍に見積もる
        let offset = from_last_sampling_sec * 1.1;
        if used + offset > self.time_limit_sec {
            let path = format!("{:>03}.png", self.report_image_counter);
            println!("reached time limit");
            println!("output final image: {}", path);
            println!("remain: {:.3} sec.", self.time_limit_sec - used);
            Self::save_progress_image(&path, accumulation_buf, sampling, imgbuf);
            return true;
        }

        // reached max sampling
        if sampling >= self.max_sampling() {
            let path = format!("{:>03}.png", self.report_image_counter);
            println!("reached max sampling");
            println!("output final image: {}", path);
            println!("remain: {:.3} sec.", self.time_limit_sec - used);
            Self::save_progress_image(&path, accumulation_buf, sampling, imgbuf);
            return true;
        }

        // on interval time passed
        let from_last_report_image_sec = (now - self.last_report_image).num_milliseconds() as f64 * 0.001;
        if from_last_report_image_sec >= self.report_interval_sec {
            // save progress image
            let path = format!("{:>03}.png", self.report_image_counter);
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
    pub fn new(sampling: u32, time_limit_sec: f64, report_interval_sec: f64) -> PathTracingRenderer {
        let now = time::now();
        PathTracingRenderer {
            sampling,
            time_limit_sec,
            report_interval_sec,

            begin: now,
            last_report_progress: now,
            last_report_image: now,
            report_image_counter: 0,
        }
    }

    fn next_event_estimation(random: (f64, f64), position: &Vector3, view: &Vector3, normal: &Vector3,
                             scene: &SceneTrait, emissions: &Vec<&Box<Intersectable>>, material: &PointMaterial) -> (Vector3, f64) {
        let mut accumulation = Vector3::zero();
        let mut bsdf_mis_weight_sum = 0.0;

        for emission in emissions {
            let surface = emission.sample_on_surface(random);
            let shadow_vec = surface.position - *position;
            let shadow_dir = shadow_vec.normalize();
            let shadow_ray = Ray { origin: *position, direction: shadow_dir };
            let (shadow_hit, shadow_intersection) = scene.intersect(&shadow_ray);

            if shadow_hit && shadow_intersection.position.approximately(&surface.position) {
                let cos_shadow = normal.dot(&shadow_dir).abs();
                let cos_light = surface.normal.dot(&shadow_dir).abs();
                let distance_pow2 = shadow_vec.dot(&shadow_vec);
                let g = (cos_shadow * cos_light) / distance_pow2;

                let light_pdf = surface.pdf;

                let fs = material.bsdf(view, normal, &shadow_dir);
                let bsdf_pdf = fs * cos_light / distance_pow2;// 単位を light_pdf に合わせる

                let light_mis_weight = light_pdf / (bsdf_pdf + light_pdf);
                bsdf_mis_weight_sum += bsdf_pdf / (bsdf_pdf + light_pdf);

                accumulation += shadow_intersection.material.emission
                    * fs * g / light_pdf * light_mis_weight;
            }
        }

        (accumulation * material.albedo, bsdf_mis_weight_sum)
    }
}
