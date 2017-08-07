use consts;
use vector::{Vector3, Vector2};
use material::Material;
use texture::Texture;

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

pub struct Intersection {
    pub position: Vector3,
    pub distance: f64,
    pub normal: Vector3,
    pub uv: Vector2,
}

impl Intersection {
    pub fn new() -> Intersection {
        Intersection {
            position: Vector3::zero(),
            distance: consts::INF,
            normal: Vector3::zero(),
            uv: Vector2::zero(),
        }
    }
}

pub trait Intersectable: Sync {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn material(&self) -> &Material;
}

pub struct Sphere {
    pub center: Vector3,
    pub radius: f64,
    pub material: Material,
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let a : Vector3 = ray.origin - self.center;
        let b = a.dot(&ray.direction);
        let c = a.dot(&a) - self.radius * self.radius;
        let d = b * b - c;
        let t = -b - d.sqrt();
        if d > 0.0 && t > 0.0 {
            let position = ray.origin + ray.direction * t;
            Some(Intersection {
                position: position,
                distance: t,
                normal: (position - self.center).normalize(),
                uv: Vector2::zero(),
            })
        } else {
            None
        }
    }

    fn material(&self) -> &Material {
        &self.material
    }
}

pub struct Plane {
    pub center: Vector3,
    pub normal: Vector3,
    pub material: Material,
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let d = -self.center.dot(&self.normal);
        let v = ray.direction.dot(&self.normal);
        let t = -(ray.origin.dot(&self.normal) + d) / v;
        if t > 0.0 {
            let position = ray.origin + ray.direction * t;
            Some(Intersection {
                position: position,
                normal: self.normal,
                distance: t,
                // normalがY軸なことを前提にUVを計算
                uv: Vector2::new(position.x % 1.0, position.z % 1.0),
            })
        } else {
            None
        }
    }

    fn material(&self) -> &Material {
        &self.material
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
    pub px_texture: Texture,
    pub nx_texture: Texture,
    pub py_texture: Texture,
    pub ny_texture: Texture,
    pub pz_texture: Texture,
    pub nz_texture: Texture,
}

impl Skybox {
    pub fn new(px_path: &str, nx_path: &str, py_path: &str, ny_path: &str, pz_path: &str, nz_path: &str) -> Skybox {
        Skybox {
            px_texture: Texture::new(px_path),
            nx_texture: Texture::new(nx_path),
            py_texture: Texture::new(py_path),
            ny_texture: Texture::new(ny_path),
            pz_texture: Texture::new(pz_path),
            nz_texture: Texture::new(nz_path),
        }
    }

    pub fn sample(&self, direction: &Vector3) -> Vector3 {
        let abs_x = direction.x.abs();
        let abs_y = direction.y.abs();
        let abs_z = direction.z.abs();

        if abs_x > abs_y && abs_x > abs_z {
            if direction.x.is_sign_positive() {
                self.px_texture.sample_bilinear_0center(-direction.z / direction.x, -direction.y / direction.x)
            } else {
                self.nx_texture.sample_bilinear_0center(-direction.z / direction.x, direction.y / direction.x)
            }
        } else if abs_y > abs_x && abs_y > abs_z {
            if direction.y.is_sign_positive() {
                self.py_texture.sample_bilinear_0center(direction.x / direction.y, direction.z / direction.y)
            } else {
                self.ny_texture.sample_bilinear_0center(-direction.x / direction.y, direction.z / direction.y)
            }
        } else {
            if direction.z.is_sign_positive() {
                self.pz_texture.sample_bilinear_0center(direction.x / direction.z, -direction.y / direction.z)
            } else {
                self.nz_texture.sample_bilinear_0center(direction.x / direction.z, direction.y / direction.z)
            }
        }
    }
}

pub struct Scene {
    pub elements: Vec<Box<Intersectable>>,
    pub skybox: Skybox,
}

impl Scene {
    // (bool, Intersection, Vector3, Vector3)
    // Option<(Option<Intersection>, &Material)>
    pub fn intersect(&self, ray: &Ray) -> (Option<Intersection>, Vector3, Vector3, Option<&Material>) {
        /*let mut intersection = Intersection::new(self.elements[0]);
        for element in &self.elements {
            element.intersect(&ray, &mut intersection);
        }
        if !intersection.hit {
            intersection.material.emission = self.skybox.sample(&ray.direction);
        }
        intersection*/

        let nearest = self.elements.iter()
            .map(|e| (e.intersect(ray), e.material()))
            .filter(|t| match t.0 {
                Some(ref intersection) => true,
                None => false
            })
            .min_by(|t1, t2| t1.0.unwrap().distance.partial_cmp(&t2.0.unwrap().distance).unwrap());

        match nearest {
            Some(tuple) => {
                let material = tuple.1;
                (tuple.0, material.albedo, material.emission, Some(material))
            },
            None => {
                (None, Vector3::zero(), self.skybox.sample(&ray.direction), None)
            }
        }
    }
}
