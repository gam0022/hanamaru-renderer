extern crate image;

use image::{DynamicImage, GenericImage};
use std::fs::File;
use std::path::Path;

use consts;
use vector::{Vector3, Vector2};
use material::Material;
use color;

#[derive(Clone, Debug)]
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
    pub material: Material,
}

impl Intersection {
    pub fn new() -> Intersection {
        Intersection {
            hit: false,
            position: Vector3::zero(),
            distance: consts::INF,
            normal: Vector3::zero(),
            material: Material::new(),
        }
    }
}

pub trait Intersectable: Sync {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection);
}

pub struct Sphere {
    pub center: Vector3,
    pub radius: f64,
    pub material: Material,
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
            intersection.material = self.material.clone();
        }
    }
}

pub struct Plane {
    pub center: Vector3,
    pub normal: Vector3,
    pub material: Material,
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
            intersection.material = self.material.clone();
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

    pub fn shoot_ray(&self, normalized_coord: &Vector2) -> Ray {
        Ray {
            origin: self.eye,
            direction: (normalized_coord.x * self.right + normalized_coord.y * self.up + self.zoom * self.forward).normalize(),
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

pub struct Skybox {
    pub px_image: DynamicImage,
    pub nx_image: DynamicImage,
    pub py_image: DynamicImage,
    pub ny_image: DynamicImage,
    pub pz_image: DynamicImage,
    pub nz_image: DynamicImage,
}

impl Skybox {
    pub fn new(px_path: &str, nx_path: &str, py_path: &str, ny_path: &str, pz_path: &str, nz_path: &str) -> Skybox {
        Skybox {
            px_image: image::open(&Path::new(px_path)).unwrap(),
            nx_image: image::open(&Path::new(nx_path)).unwrap(),
            py_image: image::open(&Path::new(py_path)).unwrap(),
            ny_image: image::open(&Path::new(ny_path)).unwrap(),
            pz_image: image::open(&Path::new(pz_path)).unwrap(),
            nz_image: image::open(&Path::new(nz_path)).unwrap(),
        }
    }

    pub fn trace(&self, direction: &Vector3) -> Vector3 {
        let abs_x = direction.x.abs();
        let abs_y = direction.y.abs();
        let abs_z = direction.z.abs();

        if abs_x > abs_y && abs_x > abs_z {
            if direction.x.is_positive() {
                self.get_color(&self.px_image, direction.z / direction.x, -direction.y / direction.x)
            } else {
                self.get_color(&self.nx_image, direction.z / direction.x, direction.y / direction.x)
            }
        } else if abs_y > abs_x && abs_y > abs_z {
            if direction.y.is_positive() {
                self.get_color(&self.py_image, direction.z / direction.y, direction.x / direction.y)
            } else {
                self.get_color(&self.ny_image, -direction.z / direction.y, -direction.x / direction.y)
            }
        } else {
            if direction.z.is_positive() {
                self.get_color(&self.pz_image, -direction.x / direction.z, -direction.y / direction.z)
            } else {
                self.get_color(&self.nz_image, -direction.x / direction.z, direction.y / direction.z)
            }
        }
    }

    fn get_color(&self, image: &DynamicImage, u: f64, v: f64) -> Vector3 {
        color::rgba_to_vector3(image.get_pixel((image.width() as f64 * (u + 1.0) * 0.5) as u32, (image.height() as f64 * (v + 1.0) * 0.5) as u32))
    }
}

pub struct Scene {
    pub elements: Vec<Box<Intersectable>>,
    pub skybox: Skybox,
}

impl Scene {
    pub fn intersect(&self, ray: &Ray) -> Intersection {
        let mut intersection = Intersection::new();
        for element in &self.elements {
            element.intersect(&ray, &mut intersection);
        }
        if !intersection.hit {
            intersection.material.emission = self.skybox.trace(&ray.direction);
        }
        intersection
    }
}