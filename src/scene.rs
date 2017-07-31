use vector::{Vector3, Vector2};
use consts;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

#[derive(Debug)]
pub struct Intersection {
    pub hit: bool,
    pub position: Vector3,
    pub distance: f64,
    pub normal: Vector3,
}

impl Intersection {
    pub fn new() -> Intersection {
        Intersection {
            hit: false,
            position: Vector3::zero(),
            distance: consts::INF,
            normal: Vector3::zero(),
        }
    }
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection);
}

pub struct Sphere {
    pub center: Vector3,
    pub radius: f64,
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection) {
        let a : Vector3 = ray.origin - self.center;
        let b = a.dot(&ray.direction);
        let c = a.dot(&a) - self.radius * self.radius;
        let d = b * b - c;
        let t = -b - d.sqrt();
        if d > 0.0 && t > 0.0 && t < intersection.distance {
            intersection.hit = true;
            intersection.position = ray.origin + ray.direction * t;
            intersection.distance = t;
            intersection.normal = (intersection.position - self.center).normalize();
        }
    }
}

pub struct Plane {
    pub center: Vector3,
    pub normal: Vector3,
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection) {
        let d = -self.center.dot(&self.normal);
        let v = ray.direction.dot(&self.normal);
        let t = -(ray.origin.dot(&self.normal) + d) / v;
        if t > 0.0 && t < intersection.distance {
            intersection.hit = true;
            intersection.position = ray.origin + ray.direction * t;
            intersection.normal = self.normal;
            intersection.distance = t;
        }
    }
}

#[derive(Debug)]
pub struct Camera {
    pub eye : Vector3,
    pub forward : Vector3,
    pub right : Vector3,
    pub up : Vector3,
    pub zoom : f64,
}

impl Camera {
    pub fn new(eye: Vector3, target: Vector3, y_up: Vector3, zoom: f64) -> Camera {
        let forward = (target - eye).normalize();
        let right = forward.cross(&y_up).normalize();

        Camera {
            eye: eye,
            forward: forward,
            right: right,
            up: right.cross(&forward).normalize(),
            zoom: zoom,
        }
    }

    pub fn shoot_ray(&self, uv: &Vector2) -> Ray {
        Ray {
            origin: self.eye,
            direction: (uv.x * self.right + uv.y * self.up + self.zoom * self.forward).normalize(),
        }
    }
}

pub struct CameraBuilder {
    eye: Vector3,
    target: Vector3,
    y_up: Vector3,
    zoom: f64,
}

impl CameraBuilder {
    pub fn new() -> CameraBuilder {
        CameraBuilder {
            eye: Vector3::zero(),
            target: Vector3::new(0.0, 0.0, 1.0),
            y_up: Vector3::new(0.0, 1.0, 0.0),
            zoom: 2.0,
        }
    }

    pub fn eye(&mut self, coordinate: Vector3) -> &mut CameraBuilder {
        self.eye = coordinate;
        self
    }

    pub fn target(&mut self, coordinate: Vector3) -> &mut CameraBuilder {
        self.target = coordinate;
        self
    }

    pub fn y_up(&mut self, coordinate: Vector3) -> &mut CameraBuilder {
        self.y_up = coordinate;
        self
    }

    pub fn zoom(&mut self, coordinate: f64) -> &mut CameraBuilder {
        self.zoom = coordinate;
        self
    }

    pub fn finalize(&self) -> Camera {
        Camera::new(self.eye, self.target, self.y_up, self.zoom)
    }
}

pub struct Scene {
    pub elements: Vec<Box<Intersectable>>,
}

impl Scene {
    pub fn intersect(&self, ray: &Ray) -> Intersection {
        let mut intersection = Intersection::new();
        for element in &self.elements {
            element.intersect(&ray, &mut intersection);
        }
        intersection
    }
}

#[allow(dead_code)]
pub fn test() {
    let ray = Ray{origin: Vector3{x: 0.0, y: 0.0, z: -3.0}, direction: Vector3{x: 0.0, y: 0.0, z: 1.0}};
    let sphere = Sphere{center: Vector3{x: 0.0, y: 0.0, z: 0.0}, radius: 1.0};
    let mut intersection = Intersection::new();
    sphere.intersect(&ray, &mut intersection);
    println!("{:?}", intersection);

    let sphere = Sphere{center: Vector3{x: 0.0, y: 0.0, z: 0.0}, radius: 2.0};
    sphere.intersect(&ray, &mut intersection);
    println!("{:?}", intersection);

    println!("{}", consts::EPS);

    let v1 = Vector3{x: 1.0, y: 2.0, z: 3.0};
    let v2 = Vector3{x: 2.0, y: 2.0, z: 3.0};
    let v3 = v1 + v2;
    println!("{:?}", v3);
}
