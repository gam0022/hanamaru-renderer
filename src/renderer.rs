use consts;
use vector::{Vector3, Vector2};
use scene::{Camera, Ray, Intersectable, Sphere, Intersection};

pub trait Renderer {
    //fn calc_radiance(ray: &mut Ray) -> Vector3;
}

pub struct DebugRenderer {
}

impl DebugRenderer {
    //fn calc_radiance(ray: &mut Ray) -> Vector3 {
    pub fn test(camera: &Camera, uv: &Vector2) -> Vector3 {
       let sphere = Sphere{ center: Vector3::new(0.0, 0.0, 0.0), radius: 1.0 };
       let ray = camera.shoot_ray(&uv);
       //println!("{:?}", ray);

       let mut intersection = Intersection::new();
       sphere.intersect(&ray, &mut intersection);
       //println!("{:?}", intersection);
       if intersection.hit {
           Vector3::new(0.0, 0.0, 0.0)
       } else {
           Vector3::new(1.0, 1.0, 1.0)
       }
   }
}
