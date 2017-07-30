use consts;
use vector::{Vector3, Vector2};
use scene::{Scene, Camera, Ray, Intersectable, Sphere, Intersection};

pub trait Renderer {
    //fn calc_radiance(ray: &mut Ray) -> Vector3;
}

pub struct DebugRenderer {
}

impl DebugRenderer {
    //fn calc_radiance(ray: &mut Ray) -> Vector3 {
    pub fn test(scene: &Scene, camera: &Camera, uv: &Vector2) -> Vector3 {
       let ray = camera.shoot_ray(&uv);
       let intersection = scene.intersect(&ray);

       if intersection.hit {
           Vector3::new(0.0, 0.0, 0.0)
       } else {
           Vector3::new(1.0, 1.0, 1.0)
       }
   }
}
