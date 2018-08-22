use config;
use vector::{Vector3, Vector2};
use material::{Material, PointMaterial, SurfaceType};
use camera::Ray;
use texture::ImageTexture;
use math::{equals_eps, modulo};
use color::Color;
use bvh::{BvhNode, Aabb, intersect_polygon};

#[derive(Debug)]
pub struct Intersection {
    pub position: Vector3,
    pub distance: f64,
    pub normal: Vector3,
    pub uv: Vector2,
    pub material: PointMaterial,
}

pub struct Surface {
    pub position: Vector3,
    pub normal: Vector3,
    pub pdf: f64,
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
    fn aabb(&self) -> Aabb;

    fn nee_available(&self) -> bool;
    fn sample_on_surface(&self, random: (f64, f64)) -> Surface;
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

    fn aabb(&self) -> Aabb {
        Aabb {
            min: self.center - Vector3::from_one(self.radius),
            max: self.center + Vector3::from_one(self.radius),
        }
    }

    fn nee_available(&self) -> bool { true }

    fn sample_on_surface(&self, random: (f64, f64)) -> Surface {
        let r1 = config::PI2 * random.0;
        let r2 = 1.0 - 2.0 * random.1;
        let r3 = (1.0 - r2 * r2).sqrt();

        // TODO: normalize が必要か確認する
        let normal = (Vector3::new(r3 * r1.cos(), r3 * r1.sin(), r2)).normalize();
        let position = self.center + (self.radius + config::OFFSET) * normal;
        let pdf = (4.0 * config::PI * self.radius * self.radius).recip();
        Surface{ position, normal, pdf }
    }
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

    // dummy method
    fn aabb(&self) -> Aabb {
        Aabb {
            min: Vector3::zero(),
            max: Vector3::zero(),
        }
    }

    fn nee_available(&self) -> bool { false }

    fn sample_on_surface(&self, random: (f64, f64)) -> Surface {
        unimplemented!()
    }
}

pub struct Cuboid {
    pub aabb: Aabb,
    pub material: Material,
}

impl Intersectable for Cuboid {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection) -> bool {
        let (hit, distance) = self.aabb.intersect_ray(ray);
        if hit && distance < intersection.distance {
            intersection.position = ray.origin + ray.direction * distance;
            intersection.distance = distance;
            let uvw = (intersection.position - self.aabb.min) / (self.aabb.max - self.aabb.min);
            // 交点座標から法線を求める
            // 高速化のためにY軸から先に判定する
            if equals_eps(intersection.position.y, self.aabb.max.y) {
                intersection.normal = Vector3::new(0.0, 1.0, 0.0);
                intersection.uv = uvw.xz();
            } else if equals_eps(intersection.position.y, self.aabb.min.y) {
                intersection.normal = Vector3::new(0.0, -1.0, 0.0);
                intersection.uv = uvw.xz();
            } else if equals_eps(intersection.position.x, self.aabb.min.x) {
                intersection.normal = Vector3::new(-1.0, 0.0, 0.0);
                intersection.uv = uvw.zy();
            } else if equals_eps(intersection.position.x, self.aabb.max.x) {
                intersection.normal = Vector3::new(1.0, 0.0, 0.0);
                intersection.uv = uvw.zy();
            } else if equals_eps(intersection.position.z, self.aabb.min.z) {
                intersection.normal = Vector3::new(0.0, 0.0, -1.0);
                intersection.uv = uvw.xy();
            } else if equals_eps(intersection.position.z, self.aabb.max.z) {
                intersection.normal = Vector3::new(0.0, 0.0, 1.0);
                intersection.uv = uvw.xy();
            }
            true
        } else {
            false
        }
    }

    fn material(&self) -> &Material { &self.material }

    fn aabb(&self) -> Aabb { self.aabb.clone() }

    fn nee_available(&self) -> bool { false }

    fn sample_on_surface(&self, random: (f64, f64)) -> Surface {
        unimplemented!()
    }
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

    // dummy method
    fn aabb(&self) -> Aabb {
        Aabb {
            min: Vector3::zero(),
            max: Vector3::zero(),
        }
    }

    fn nee_available(&self) -> bool { false }

    fn sample_on_surface(&self, random: (f64, f64)) -> Surface {
        unimplemented!()
    }
}

pub struct BvhMesh {
    pub mesh: Mesh,
    pub bvh: BvhNode,
}

impl Intersectable for BvhMesh {
    fn intersect(&self, ray: &Ray, intersection: &mut Intersection) -> bool {
        self.bvh.intersect_for_mesh(&self.mesh, ray, intersection)
    }

    fn material(&self) -> &Material { &self.mesh.material }

    fn aabb(&self) -> Aabb { self.bvh.aabb.clone() }

    fn nee_available(&self) -> bool { false }

    fn sample_on_surface(&self, random: (f64, f64)) -> Surface {
        unimplemented!()
    }
}

impl BvhMesh {
    pub fn from_mesh(mesh: Mesh) -> BvhMesh {
        let bvh = BvhNode::build_from_mesh(&mesh);
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
    pub intensity: Vector3,
}

impl Skybox {
    pub fn new(px_path: &str, nx_path: &str, py_path: &str, ny_path: &str, pz_path: &str, nz_path: &str, intensity: &Vector3) -> Skybox {
        Skybox {
            px_texture: ImageTexture::new(px_path),
            nx_texture: ImageTexture::new(nx_path),
            py_texture: ImageTexture::new(py_path),
            ny_texture: ImageTexture::new(ny_path),
            pz_texture: ImageTexture::new(pz_path),
            nz_texture: ImageTexture::new(nz_path),
            intensity: *intensity,
        }
    }

    pub fn one(px_path: &str, nx_path: &str, py_path: &str, ny_path: &str, pz_path: &str, nz_path: &str) -> Skybox {
        Skybox::new(px_path, nx_path, py_path, ny_path, pz_path, nz_path, &Vector3::one())
    }

    pub fn sample(&self, direction: &Vector3) -> Vector3 {
        let abs_x = direction.x.abs();
        let abs_y = direction.y.abs();
        let abs_z = direction.z.abs();

        if abs_x > abs_y && abs_x > abs_z {
            if direction.x.is_sign_positive() {
                self.intensity * self.px_texture.sample_bilinear_0center(-direction.z / direction.x, direction.y / direction.x)
            } else {
                self.intensity * self.nx_texture.sample_bilinear_0center(-direction.z / direction.x, -direction.y / direction.x)
            }
        } else if abs_y > abs_x && abs_y > abs_z {
            if direction.y.is_sign_positive() {
                self.intensity * self.py_texture.sample_bilinear_0center(direction.x / direction.y, -direction.z / direction.y)
            } else {
                self.intensity * self.ny_texture.sample_bilinear_0center(-direction.x / direction.y, -direction.z / direction.y)
            }
        } else {
            if direction.z.is_sign_positive() {
                self.intensity * self.pz_texture.sample_bilinear_0center(direction.x / direction.z, direction.y / direction.z)
            } else {
                self.intensity * self.nz_texture.sample_bilinear_0center(direction.x / direction.z, -direction.y / direction.z)
            }
        }
    }
}

pub trait SceneTrait: Sync {
    fn intersect(&self, ray: &Ray) -> (bool, Intersection);
    fn emissions(&self) -> Vec<&Box<Intersectable>>;
}

pub struct Scene {
    pub elements: Vec<Box<Intersectable>>,
    pub skybox: Skybox,
}

impl SceneTrait for Scene {
    fn intersect(&self, ray: &Ray) -> (bool, Intersection) {
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

    fn emissions(&self) -> Vec<&Box<Intersectable>> {
        self.elements.iter().filter(|e| e.nee_available() && e.material().emission.color != Color::zero()).collect()
    }
}

impl Scene {
    pub fn add(&mut self, element: Box<Intersectable>) {
        self.elements.push(element);
    }

    pub fn add_with_check_collisions(&mut self, element: Box<Intersectable>) -> bool {
        let aabb = element.aabb();
        let no_collisions = self.elements.iter().all(|ref e| !e.aabb().intersect_aabb(&aabb));
        if no_collisions {
            self.elements.push(element);
            true
        } else {
            //println!("add_with_check_collisions: collisions!: {:?}", aabb);
            false
        }
    }
}

pub struct BvhScene {
    pub scene: Scene,
    pub bvh: BvhNode,
}

impl SceneTrait for BvhScene {
    fn intersect(&self, ray: &Ray) -> (bool, Intersection) {
        let mut intersection = Intersection::empty();
        let nearest_index = self.bvh.intersect_for_scene(&self.scene, ray, &mut intersection);

        if let Some(index) = nearest_index {
            let element = &self.scene.elements[index];
            let material = element.material();
            intersection.material.surface = material.surface.clone();
            intersection.material.albedo = material.albedo.sample(intersection.uv);
            intersection.material.emission = material.emission.sample(intersection.uv);
            intersection.material.roughness = material.roughness.sample(intersection.uv).x;
            (true, intersection)
        } else {
            intersection.material.emission = self.scene.skybox.sample(&ray.direction);
            (false, intersection)
        }
    }

    fn emissions(&self) -> Vec<&Box<Intersectable>> {
        self.scene.emissions()
    }
}

impl BvhScene {
    pub fn from_scene(scene: Scene) -> BvhScene {
        let bvh = BvhNode::build_from_scene(&scene);
        BvhScene {
            scene: scene,
            bvh: bvh,
        }
    }
}
