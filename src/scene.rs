use config;
use vector::{Vector3, Vector2};
use material::{Material, PointMaterial, SurfaceType};
use camera::Ray;
use texture::ImageTexture;
use math::{equals_eps, modulo};
use color::Color;
use bvh::{BvhNode, intersect_polygon};

#[derive(Debug)]
pub struct Intersection {
    pub position: Vector3,
    pub distance: f64,
    pub normal: Vector3,
    pub uv: Vector2,
    pub material: PointMaterial,
}

impl Intersection {
    pub fn empty() -> Intersection {
        Intersection {
            position: Vector3::zero(),
            distance: config::INF,
            normal: Vector3::zero(),
            uv: Vector2::zero(),
            material: PointMaterial {
                surface: SurfaceType::Diffuse,
                albedo: Color::one(),
                emission: Color::zero(),
                roughness: 0.2,
            },
        }
    }
}

pub trait Intersectable: Sync {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection) -> bool;
    fn material(&self) -> &Material;
}

pub struct Sphere {
    pub center: Vector3,
    pub radius: f64,
    pub material: Material,
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection) -> bool {
        let a: Vector3 = ray.origin - self.center;
        let b = a.dot(&ray.direction);
        let c = a.dot(&a) - self.radius * self.radius;
        let d = b * b - c;
        let t = -b - d.sqrt();
        if d > 0.0 && t > 0.0 && t < intersection.distance {
            intersection.position = ray.origin + ray.direction * t;
            intersection.distance = t;
            intersection.normal = (intersection.position - self.center).normalize();

            intersection.uv.y = 1.0 - intersection.normal.y.acos() / config::PI;
            intersection.uv.x = 0.5
                - intersection.normal.z.signum()
                * (intersection.normal.x / intersection.normal.xz().length()).acos()
                / config::PI2;
            true
        } else {
            false
        }
    }

    fn material(&self) -> &Material { &self.material }
}

#[allow(dead_code)]
pub struct Plane {
    pub center: Vector3,
    pub normal: Vector3,
    pub material: Material,
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection) -> bool {
        let d = -self.center.dot(&self.normal);
        let v = ray.direction.dot(&self.normal);
        let t = -(ray.origin.dot(&self.normal) + d) / v;
        if t > 0.0 && t < intersection.distance {
            intersection.position = ray.origin + ray.direction * t;
            intersection.normal = self.normal;
            intersection.distance = t;

            // normalがY軸なことを前提にUVを計算
            intersection.uv = Vector2::new(modulo(intersection.position.x, 1.0), modulo(intersection.position.z, 1.0));
            true
        } else {
            false
        }
    }

    fn material(&self) -> &Material { &self.material }
}

pub struct AxisAlignedBoundingBox {
    pub left_bottom: Vector3,
    pub right_top: Vector3,
    pub material: Material,
}

impl Intersectable for AxisAlignedBoundingBox {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection) -> bool {
        let dir_inv = Vector3::new(
            ray.direction.x.recip(),
            ray.direction.y.recip(),
            ray.direction.z.recip(),
        );

        let t1 = (self.left_bottom.x - ray.origin.x) * dir_inv.x;
        let t2 = (self.right_top.x - ray.origin.x) * dir_inv.x;
        let t3 = (self.left_bottom.y - ray.origin.y) * dir_inv.y;
        let t4 = (self.right_top.y - ray.origin.y) * dir_inv.y;
        let t5 = (self.left_bottom.z - ray.origin.z) * dir_inv.z;
        let t6 = (self.right_top.z - ray.origin.z) * dir_inv.z;
        let tmin = (t1.min(t2).max(t3.min(t4))).max(t5.min(t6));
        let tmax = (t1.max(t2).min(t3.max(t4))).min(t5.max(t6));
        let distance = if tmin.is_sign_positive() { tmin } else { tmax };

        if tmin <= tmax && tmax.is_sign_positive() && distance < intersection.distance {
            intersection.position = ray.origin + ray.direction * distance;
            intersection.distance = distance;
            let uvw = (intersection.position - self.left_bottom) / (self.right_top - self.left_bottom);
            // 交点座標から法線を求める
            // 高速化のためにY軸から先に判定する
            if equals_eps(intersection.position.y, self.right_top.y) {
                intersection.normal = Vector3::new(0.0, 1.0, 0.0);
                intersection.uv = uvw.xz();
            } else if equals_eps(intersection.position.y, self.left_bottom.y) {
                intersection.normal = Vector3::new(0.0, -1.0, 0.0);
                intersection.uv = uvw.xz();
            } else if equals_eps(intersection.position.x, self.left_bottom.x) {
                intersection.normal = Vector3::new(-1.0, 0.0, 0.0);
                intersection.uv = uvw.zy();
            } else if equals_eps(intersection.position.x, self.right_top.x) {
                intersection.normal = Vector3::new(1.0, 0.0, 0.0);
                intersection.uv = uvw.zy();
            } else if equals_eps(intersection.position.z, self.left_bottom.z) {
                intersection.normal = Vector3::new(0.0, 0.0, -1.0);
                intersection.uv = uvw.xy();
            } else if equals_eps(intersection.position.z, self.right_top.z) {
                intersection.normal = Vector3::new(0.0, 0.0, 1.0);
                intersection.uv = uvw.xy();
            }
            true
        } else {
            false
        }
    }

    fn material(&self) -> &Material { &self.material }
}

pub struct Face {
    pub v0: usize,
    pub v1: usize,
    pub v2: usize,
}

pub struct Mesh {
    pub vertexes: Vec<Vector3>,
    pub faces: Vec<Face>,
    pub material: Material,
}

impl Intersectable for Mesh {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection) -> bool {
        let mut any_hit = false;
        for face in &self.faces {
            if intersect_polygon(&self.vertexes[face.v0], &self.vertexes[face.v1], &self.vertexes[face.v2], ray, intersection) {
                any_hit = true;
            }
        }
        any_hit
    }

    fn material(&self) -> &Material { &self.material }
}

pub struct BvhMesh {
    pub mesh: Mesh,
    pub bvh: BvhNode,
}

impl Intersectable for BvhMesh {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection) -> bool {
        self.bvh.intersect(&self.mesh, ray, intersection)
    }

    fn material(&self) -> &Material { &self.mesh.material }
}

impl BvhMesh {
    pub fn from_mesh(mesh: Mesh) -> BvhMesh {
        let bvh = BvhNode::from_mesh(&mesh);
        //println!("bvh: {:?}", bvh);
        BvhMesh {
            bvh: bvh,
            mesh: mesh,
        }
    }
}

pub struct Skybox {
    pub px_texture: ImageTexture,
    pub nx_texture: ImageTexture,
    pub py_texture: ImageTexture,
    pub ny_texture: ImageTexture,
    pub pz_texture: ImageTexture,
    pub nz_texture: ImageTexture,
}

impl Skybox {
    pub fn new(px_path: &str, nx_path: &str, py_path: &str, ny_path: &str, pz_path: &str, nz_path: &str) -> Skybox {
        Skybox {
            px_texture: ImageTexture::new(px_path),
            nx_texture: ImageTexture::new(nx_path),
            py_texture: ImageTexture::new(py_path),
            ny_texture: ImageTexture::new(ny_path),
            pz_texture: ImageTexture::new(pz_path),
            nz_texture: ImageTexture::new(nz_path),
        }
    }

    pub fn sample(&self, direction: &Vector3) -> Vector3 {
        let abs_x = direction.x.abs();
        let abs_y = direction.y.abs();
        let abs_z = direction.z.abs();

        if abs_x > abs_y && abs_x > abs_z {
            if direction.x.is_sign_positive() {
                self.px_texture.sample_bilinear_0center(-direction.z / direction.x, direction.y / direction.x)
            } else {
                self.nx_texture.sample_bilinear_0center(-direction.z / direction.x, -direction.y / direction.x)
            }
        } else if abs_y > abs_x && abs_y > abs_z {
            if direction.y.is_sign_positive() {
                self.py_texture.sample_bilinear_0center(direction.x / direction.y, -direction.z / direction.y)
            } else {
                self.ny_texture.sample_bilinear_0center(-direction.x / direction.y, -direction.z / direction.y)
            }
        } else {
            if direction.z.is_sign_positive() {
                self.pz_texture.sample_bilinear_0center(direction.x / direction.z, direction.y / direction.z)
            } else {
                self.nz_texture.sample_bilinear_0center(direction.x / direction.z, -direction.y / direction.z)
            }
        }
    }
}

pub struct Scene {
    pub elements: Vec<Box<Intersectable>>,
    pub skybox: Skybox,
}

impl Scene {
    pub fn intersect(&self, ray: &Ray) -> (bool, Intersection) {
        let mut intersection = Intersection::empty();
        let mut nearest: Option<&Box<Intersectable>> = None;

        for e in &self.elements {
            if e.intersect(&ray, &mut intersection) {
                nearest = Some(&e);
            }
        }

        if let Some(element) = nearest {
            let material = element.material();
            intersection.material.surface = material.surface.clone();
            intersection.material.albedo = material.albedo.sample(intersection.uv);
            intersection.material.emission = material.emission.sample(intersection.uv);
            intersection.material.roughness = material.roughness.sample(intersection.uv).x;
            (true, intersection)
        } else {
            intersection.material.emission = self.skybox.sample(&ray.direction);
            (false, intersection)
        }
    }

    pub fn add(&mut self, element: Box<Intersectable>) {
        self.elements.push(element);
    }
}
