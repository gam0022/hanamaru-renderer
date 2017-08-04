extern crate image;
extern crate rand;

use image::{ImageBuffer, Rgb};
use self::rand::{thread_rng, Rng};

use consts;
use vector::{Vector3, Vector2};
use scene::{Scene, Camera, Ray};
use material::SurfaceType;
use brdf;
use random;

pub trait Renderer {
    fn render(&self, scene: &Scene, camera: &Camera, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let resolution = Vector2::new(imgbuf.width() as f64, imgbuf.height() as f64);
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let frag_coord = Vector2::new(x as f64, resolution.y - y as f64);
            let uv = (frag_coord * 2.0 - resolution) / resolution.x.min(resolution.y);
            let color = self.calc_pixel(&scene, &camera, &uv);
            *pixel = image::Rgb([
                (255.0 * color.x) as u8,
                (255.0 * color.y) as u8,
                (255.0 * color.z) as u8,
            ]);
        }
    }
    fn calc_pixel(&self, scene: &Scene, camera: &Camera, uv: &Vector2) -> Vector3;
}

pub struct DebugRenderer;
impl Renderer for DebugRenderer {
    fn calc_pixel(&self, scene: &Scene, camera: &Camera, uv: &Vector2) -> Vector3 {
        let mut ray = camera.shoot_ray(&uv);
        let light_direction = Vector3::new(1.0, 2.0, 1.0).normalize();

        let mut accumulation = Vector3::zero();
        let mut reflection = Vector3::one();

        for i in 1..consts::DEBUG_BOUNCE_LIMIT {
            let intersection = scene.intersect(&ray);

            let shadow_ray = Ray {
                origin: intersection.position + intersection.normal * consts::OFFSET,
                direction: light_direction,
            };
            let shadow_intersection = scene.intersect(&shadow_ray);
            let shadow = if shadow_intersection.hit { 0.5 } else { 1.0 };

            match intersection.material.surface {
                SurfaceType::Diffuse => {
                    let diffuse = intersection.normal.dot(&light_direction).max(0.0);
                    let color = intersection.material.emission + intersection.material.albedo * diffuse * shadow;
                    reflection = reflection * color;
                    accumulation = accumulation + reflection;
                    break;
                },
                SurfaceType::Specular => {
                    ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                    ray.direction = ray.direction.reflect(&intersection.normal);
                    reflection = reflection * intersection.material.albedo;
                },
                SurfaceType::Reflection { refractiveIndex: refractiveIndex } => {},
                SurfaceType::GGX { roughness: roughness } => {},
                SurfaceType::GGXReflection { refractiveIndex: refractiveIndex, roughness: roughness } => {},
            }

            if !intersection.hit {
                break;
            }
        }

        accumulation
   }
}
pub struct PathTracingRenderer;
impl Renderer for PathTracingRenderer {
    fn calc_pixel(&self, scene: &Scene, camera: &Camera, uv: &Vector2) -> Vector3 {
        let original_ray = camera.shoot_ray(&uv);
        let mut all_accumulation = Vector3::zero();
        let mut rng = thread_rng();
        for sampling in 1..consts::PATHTRACING_SAMPLING {
            let mut ray = original_ray.clone();
            let mut accumulation = Vector3::zero();
            let mut reflection = Vector3::one();

            for bounce in 1..consts::PATHTRACING_BOUNCE_LIMIT {
                let random = random::get_random(&mut rng);
                let intersection = scene.intersect(&ray);

                accumulation = accumulation + reflection * intersection.material.emission;
                reflection = reflection * intersection.material.albedo;

                if intersection.hit {
                    match intersection.material.surface {
                        SurfaceType::Diffuse => {
                            ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                            ray.direction = brdf::importance_sample_diffuse(random, intersection.normal);
                        },
                        SurfaceType::Specular => {
                            ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                            ray.direction = ray.direction.reflect(&intersection.normal);
                        },
                        SurfaceType::Reflection { refractiveIndex: refractiveIndex } => {},
                        SurfaceType::GGX { roughness: roughness } => {},
                        SurfaceType::GGXReflection { refractiveIndex: refractiveIndex, roughness: roughness } => {},
                    }
                } else {
                    break;
                }
            }
            all_accumulation = all_accumulation + accumulation;
        }

        all_accumulation / consts::PATHTRACING_SAMPLING as f64
    }
}