use vector::Vector3;
use consts;

pub struct Ray {
    pub origin : Vector3,
    pub direction : Vector3,
}

#[derive(Debug)]
pub struct Intersection {
    pub position : Vector3,
    pub distance : f64,
    pub normal : Vector3,
}

pub struct Sphere {
    pub center : Vector3,
    pub radius : f64,
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let a : Vector3 = ray.origin - self.center;
        let b = a.dot(&ray.direction);
        let c = a.dot(&a) - self.radius * self.radius;
        let d = b * b - c;
        let t = -b - d.sqrt();
        if d > 0.0 && t > 0.0 {
            Some(Intersection {
                position : ray.origin + ray.direction * t,
                distance : t,
                normal : (ray.origin + ray.direction * t - self.center).normalize(),
            })
        } else {
            None
        }
    }
}

fn aa(elem : &Intersectable, ray: &Ray) -> Option<Intersection> {
    elem.intersect(&ray)
}

pub fn test() {
    let ray = Ray{origin: Vector3{x: 0.0, y: 0.0, z: -3.0}, direction: Vector3{x: 0.0, y: 0.0, z: 1.0}};
    let sphere = Sphere{center: Vector3{x: 0.0, y: 0.0, z: 0.0}, radius: 1.0};
    let intersection = sphere.intersect(&ray);
    println!("{:?}", intersection);
    println!("{}", consts::EPS);

    let intersection = aa(&sphere, &ray);
    println!("{:?}", intersection);

    let v1 = Vector3{x: 1.0, y: 2.0, z: 3.0};
    let v2 = Vector3{x: 2.0, y: 2.0, z: 3.0};
    let v3 = v1 + v2;
    println!("{:?}", v3);
}
