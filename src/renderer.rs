extern crate image;
use image::{ImageBuffer, Rgb};

use consts;
use vector::{Vector3, Vector2};
use scene::{Scene, Camera, Ray};
use material::SurfaceType;

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
        let light_direction = Vector3::new(1.0, 2.0, 1.0).normalize();
        let mut color = Vector3::from_one(1.0);
        let mut ray = camera.shoot_ray(&uv);

        for i in 1..5 {
            let intersection = scene.intersect(&ray);

            let diffuse = intersection.normal.dot(&light_direction).max(0.0);
            let shadow_ray = Ray {
                origin: intersection.position + intersection.normal * consts::OFFSET,
                direction: light_direction,
            };
            let shadow_intersection = scene.intersect(&shadow_ray);
            let shadow = if shadow_intersection.hit { 0.5 } else { 1.0 };

            color = color * (intersection.material.emission + intersection.material.albedo * diffuse * shadow);

            if !intersection.hit {
                break;
            }

            match intersection.material.surface {
                SurfaceType::Diffuse => break,
                SurfaceType::Specular => {
                    ray.origin = intersection.position + intersection.normal * consts::OFFSET;
                    ray.direction = ray.direction.reflect(&intersection.normal);
                },
                SurfaceType::Reflection { refractiveIndex: refractiveIndex } => {},
                SurfaceType::GGX { roughness: roughness } => {},
                SurfaceType::GGXReflection { refractiveIndex: refractiveIndex, roughness: roughness } => {},
            }
        }

        color
   }
}
