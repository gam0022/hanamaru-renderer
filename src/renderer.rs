extern crate image;
extern crate rand;
extern crate rayon;

use image::{ImageBuffer, Rgb};
use self::rand::thread_rng;
use self::rayon::prelude::*;

use consts;
use vector::{Vector3, Vector2};
use scene::{Scene, Camera, Ray};
use material::SurfaceType;
use brdf;
use random;
use color::{Color, color_to_rgb};

pub trait Renderer: Sync {
    fn render_single_thread(&self, scene: &Scene, camera: &Camera, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let resolution = Vector2::new(imgbuf.width() as f64, imgbuf.height() as f64);
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let frag_coord = Vector2::new(x as f64, resolution.y - y as f64);
            *pixel = color_to_rgb(self.supersampling(scene, camera, &frag_coord, &resolution));
        }
    }

    fn render(&self, scene: &Scene, camera: &Camera, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let resolution = Vector2::new(imgbuf.width() as f64, imgbuf.height() as f64);
        for y in 0..imgbuf.height() {
            let input: Vec<u32> = (0..imgbuf.width()).collect();
            let mut output = vec![];
            input.par_iter()
                .map(|&x| {
                    let frag_coord = Vector2::new(x as f64, resolution.y - y as f64);
                    color_to_rgb(self.supersampling(scene, camera, &frag_coord, &resolution))
                }).collect_into(&mut output);
            for (x, pixel) in output.iter().enumerate() {
                imgbuf.put_pixel(x as u32, y, *pixel);
            }
        }
    }

    fn supersampling(&self, scene: &Scene, camera: &Camera, frag_coord: &Vector2, resolution: &Vector2) -> Color {
        let mut accumulation = Color::zero();

        for sy in 0..consts::SUPERSAMPLING {
            for sx in 0..consts::SUPERSAMPLING {
                let offset = Vector2::new(sx as f64, sy as f64) / consts::SUPERSAMPLING as f64 - 0.5;
                let normalized_coord = ((*frag_coord + offset) * 2.0 - *resolution) / resolution.x.min(resolution.y);
                let color = self.calc_pixel(&scene, &camera, &normalized_coord);
                accumulation = accumulation + color;
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

        for bounce in 1..consts::DEBUG_BOUNCE_LIMIT {
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
                        reflection = reflection * intersection.material.albedo;
                    },
                    // 鏡面以外は拡散面として処理する
                    _ => {
                        let diffuse = intersection.normal.dot(&light_direction).max(0.0);
                        let color = intersection.material.emission + intersection.material.albedo * diffuse * shadow;
                        reflection = reflection * color;
                        accumulation = accumulation + reflection;
                        break;
                    },
                }
            } else {
                reflection = reflection * intersection.material.emission;
                accumulation = accumulation + reflection;
                break;
            }
        }

        accumulation
   }
}
pub struct PathTracingRenderer;
impl Renderer for PathTracingRenderer {
    fn calc_pixel(&self, scene: &Scene, camera: &Camera, normalized_coord: &Vector2) -> Color {
        let original_ray = camera.ray(&normalized_coord);
        let mut all_accumulation = Vector3::zero();
        let mut rng = thread_rng();
        for sampling in 1..consts::PATHTRACING_SAMPLING {
            let mut ray = original_ray.clone();
            let mut accumulation = Color::zero();
            let mut reflection = Color::one();

            for bounce in 1..consts::PATHTRACING_BOUNCE_LIMIT {
                let random = random::get_random(&mut rng);
                let (hit, mut intersection) = scene.intersect(&ray);

                if hit {
                    match intersection.material.surface {
                        SurfaceType::Diffuse => {
                            ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                            ray.direction = brdf::importance_sample_diffuse(random, &intersection.normal);
                        },
                        SurfaceType::Specular => {
                            ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                            ray.direction = ray.direction.reflect(&intersection.normal);
                        },
                        SurfaceType::Refraction { refractive_index } => {
                            brdf::sample_refraction(random, &intersection.normal, refractive_index, &intersection, &mut ray);
                        },
                        SurfaceType::GGX { roughness } => {
                            let alpha2 = brdf::roughness_to_alpha2(roughness);
                            ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                            let half = brdf::importance_sample_ggx(random, &intersection.normal, alpha2);

                            let v_dot_n = -ray.direction.dot(&intersection.normal);
                            ray.direction = ray.direction.reflect(&half);
                            let l_dot_n = ray.direction.dot(&intersection.normal);
                            intersection.material.albedo = intersection.material.albedo * brdf::g_smith_joint(l_dot_n, v_dot_n, alpha2);

                            // 半球外が選ばれた場合はBRDFを0にする
                            // 真値よりも暗くなるので、サンプリングやり直す方が理想的ではありそう
                            if intersection.normal.dot(&ray.direction).is_sign_negative() {
                                intersection.material.albedo = Vector3::zero();
                            }
                        },
                        SurfaceType::GGXReflection { refractive_index, roughness } => {
                            let alpha2 = brdf::roughness_to_alpha2(roughness);
                            let half = brdf::importance_sample_ggx(random, &intersection.normal, alpha2);
                            brdf::sample_refraction(random, &half, refractive_index, &intersection, &mut ray);
                        },
                    }
                }

                accumulation = accumulation + reflection * intersection.material.emission;
                reflection = reflection * intersection.material.albedo;

                if !hit { break; }
            }
            all_accumulation = all_accumulation + accumulation;
        }

        all_accumulation / consts::PATHTRACING_SAMPLING as f64
    }
}
