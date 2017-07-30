extern crate image;
use image::{ImageBuffer, Pixel, Rgb};

use consts;
use vector::{Vector3, Vector2};
use scene::{Scene, Camera, Ray, Intersectable, Sphere, Intersection};

pub trait Renderer {
    fn calc_pixel(&self, scene: &Scene, camera: &Camera, uv: &Vector2) -> Vector3;
}

pub struct DebugRenderer;

impl DebugRenderer {
    pub fn calc_pixel(&self, scene: &Scene, camera: &Camera, uv: &Vector2) -> Vector3 {
       let ray = camera.shoot_ray(&uv);
       let intersection = scene.intersect(&ray);
       let light_direction = Vector3::new(1.0, -1.0, 1.0).normalize();

       if intersection.hit {
           let diffuse = intersection.normal.dot(&light_direction).max(0.0);
           Vector3::new(1.0, 1.0, 1.0) * diffuse
       } else {
           Vector3::new(1.0, 1.0, 1.0)
       }
   }

   pub fn render(&self, scene: &Scene, camera: &Camera, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
       let resolution = Vector2::new(imgbuf.width() as f64, imgbuf.height() as f64);
       for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
           let frag_coord = Vector2::new(x as f64, y as f64);
           let uv = (frag_coord * 2.0 - resolution) / resolution.x.min(resolution.y);
           let color = self.calc_pixel(&scene, &camera, &uv);
           *pixel = image::Rgb([
               (255.0 * color.x) as u8,
               (255.0 * color.y) as u8,
               (255.0 * color.z) as u8,
           ]);
       }
   }
}
