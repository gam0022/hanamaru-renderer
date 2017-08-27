use vector::{Vector3, Vector2};
use scene::{Mesh, Intersection, Scene};
use camera::Ray;
use config;
use math::det;

#[derive(Debug, Clone)]
pub struct Aabb {
    pub min: Vector3,
    pub max: Vector3,
}

impl Aabb {
    pub fn intersect_aabb(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x && self.max.x > other.min.x &&
            self.min.y < other.max.y && self.max.y > other.min.y &&
            self.min.z < other.max.z && self.max.z > other.min.z
    }

    pub fn intersect_ray(&self, ray: &Ray) -> (bool, f64) {
        let dir_inv = Vector3::new(
            ray.direction.x.recip(),
            ray.direction.y.recip(),
            ray.direction.z.recip(),
        );

        let t1 = (self.min.x - ray.origin.x) * dir_inv.x;
        let t2 = (self.max.x - ray.origin.x) * dir_inv.x;
        let t3 = (self.min.y - ray.origin.y) * dir_inv.y;
        let t4 = (self.max.y - ray.origin.y) * dir_inv.y;
        let t5 = (self.min.z - ray.origin.z) * dir_inv.z;
        let t6 = (self.max.z - ray.origin.z) * dir_inv.z;
        let tmin = (t1.min(t2).max(t3.min(t4))).max(t5.min(t6));
        let tmax = (t1.max(t2).min(t3.max(t4))).min(t5.max(t6));

        let hit = tmin <= tmax && tmax.is_sign_positive();
        let distance = if tmin.is_sign_positive() { tmin } else { tmax };
        (hit, distance)
    }

    pub fn merge(&mut self, other: &Aabb) {
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.min.z = self.min.z.min(other.min.z);

        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
        self.max.z = self.max.z.max(other.max.z);
    }

    pub fn from_triangle(v0: &Vector3, v1: &Vector3, v2: &Vector3) -> Aabb {
        Aabb {
            min: Vector3::new(
                v0.x.min(v1.x).min(v2.x),
                v0.y.min(v1.y).min(v2.y),
                v0.z.min(v1.z).min(v2.z),
            ),
            max: Vector3::new(
                v0.x.max(v1.x).max(v2.x),
                v0.y.max(v1.y).max(v2.y),
                v0.z.max(v1.z).max(v2.z),
            )
        }
    }
}

#[derive(Debug)]
pub struct BvhNode {
    pub aabb: Aabb,

    // size must be 0 or 2
    // empty means leaf node
    pub children: Vec<Box<BvhNode>>,

    // has faces means leaf node
    pub indexes: Vec<usize>,
}

impl BvhNode {
    fn empty() -> BvhNode {
        BvhNode {
            aabb: Aabb {
                min: Vector3::new(config::INF, config::INF, config::INF),
                max: Vector3::new(-config::INF, -config::INF, -config::INF),
            },
            children: vec![],
            indexes: vec![],
        }
    }

    fn set_aabb_from_mesh(&mut self, mesh: &Mesh, face_indexes: &Vec<usize>) {
        for face_index in face_indexes {
            let face = &mesh.faces[*face_index];
            let v0 = &mesh.vertexes[face.v0];
            let v1 = &mesh.vertexes[face.v1];
            let v2 = &mesh.vertexes[face.v2];
            self.aabb.merge(&Aabb::from_triangle(v0, v1, v2));
        }
    }

    fn set_aabb_from_scene(&mut self, scene: &Scene, indexes: &Vec<usize>) {
        for index in indexes {
            self.aabb.merge(&scene.elements[*index].aabb());
        }
    }

    fn build_from_mesh_with_indexes(mesh: &Mesh, face_indexes: &mut Vec<usize>) -> BvhNode {
        let mut node = BvhNode::empty();
        node.set_aabb_from_mesh(mesh, face_indexes);

        let mid = face_indexes.len() / 2;
        if mid <= 2 {
            // set leaf node
            node.indexes = face_indexes.clone();
        } else {
            // set intermediate node
            let lx = node.aabb.max.x - node.aabb.min.x;
            let ly = node.aabb.max.y - node.aabb.min.y;
            let lz = node.aabb.max.z - node.aabb.min.z;

            if lx > ly && lx > lz {
                face_indexes.sort_by(|a, b| {
                    let a_face = &mesh.faces[*a];
                    let b_face = &mesh.faces[*b];
                    let a_sum = mesh.vertexes[a_face.v0].x + mesh.vertexes[a_face.v1].x + mesh.vertexes[a_face.v2].x;
                    let b_sum = mesh.vertexes[b_face.v0].x + mesh.vertexes[b_face.v1].x + mesh.vertexes[b_face.v2].x;
                    a_sum.partial_cmp(&b_sum).unwrap()
                });
            } else if ly > lx && ly > lz {
                face_indexes.sort_by(|a, b| {
                    let a_face = &mesh.faces[*a];
                    let b_face = &mesh.faces[*b];
                    let a_sum = mesh.vertexes[a_face.v0].y + mesh.vertexes[a_face.v1].y + mesh.vertexes[a_face.v2].y;
                    let b_sum = mesh.vertexes[b_face.v0].y + mesh.vertexes[b_face.v1].y + mesh.vertexes[b_face.v2].y;
                    a_sum.partial_cmp(&b_sum).unwrap()
                });
            } else {
                face_indexes.sort_by(|a, b| {
                    let a_face = &mesh.faces[*a];
                    let b_face = &mesh.faces[*b];
                    let a_sum = mesh.vertexes[a_face.v0].z + mesh.vertexes[a_face.v1].z + mesh.vertexes[a_face.v2].z;
                    let b_sum = mesh.vertexes[b_face.v0].z + mesh.vertexes[b_face.v1].z + mesh.vertexes[b_face.v2].z;
                    a_sum.partial_cmp(&b_sum).unwrap()
                });
            }

            let mut left_face_indexes = face_indexes.split_off(mid);
            node.children.push(Box::new(BvhNode::build_from_mesh_with_indexes(mesh, face_indexes)));
            node.children.push(Box::new(BvhNode::build_from_mesh_with_indexes(mesh, &mut left_face_indexes)));
        }

        node
    }

    fn build_from_scene_with_indexes(scene: &Scene, indexes: &mut Vec<usize>) -> BvhNode {
        let mut node = BvhNode::empty();
        node.set_aabb_from_scene(scene, indexes);

        let mid = indexes.len() / 2;
        if mid <= 2 {
            // set leaf node
            node.indexes = indexes.clone();
        } else {
            // set intermediate node
            let lx = node.aabb.max.x - node.aabb.min.x;
            let ly = node.aabb.max.y - node.aabb.min.y;
            let lz = node.aabb.max.z - node.aabb.min.z;

            if lx > ly && lx > lz {
                indexes.sort_by(|a, b| {
                    let a_aabb = scene.elements[*a].aabb();
                    let b_aabb = scene.elements[*b].aabb();
                    let a_sum = a_aabb.min.x + a_aabb.max.x;
                    let b_sum = b_aabb.min.x + b_aabb.max.x;
                    a_sum.partial_cmp(&b_sum).unwrap()
                });
            } else if ly > lx && ly > lz {
                indexes.sort_by(|a, b| {
                    let a_aabb = scene.elements[*a].aabb();
                    let b_aabb = scene.elements[*b].aabb();
                    let a_sum = a_aabb.min.y + a_aabb.max.y;
                    let b_sum = b_aabb.min.y + b_aabb.max.y;
                    a_sum.partial_cmp(&b_sum).unwrap()
                });
            } else {
                indexes.sort_by(|a, b| {
                    let a_aabb = scene.elements[*a].aabb();
                    let b_aabb = scene.elements[*b].aabb();
                    let a_sum = a_aabb.min.z + a_aabb.max.z;
                    let b_sum = b_aabb.min.z + b_aabb.max.z;
                    a_sum.partial_cmp(&b_sum).unwrap()
                });
            }

            let mut left_face_indexes = indexes.split_off(mid);
            node.children.push(Box::new(BvhNode::build_from_scene_with_indexes(scene, indexes)));
            node.children.push(Box::new(BvhNode::build_from_scene_with_indexes(scene, &mut left_face_indexes)));
        }

        node
    }

    pub fn build_from_mesh(mesh: &Mesh) -> BvhNode {
        let mut face_indexes: Vec<usize> = (0..mesh.faces.len()).collect();
        BvhNode::build_from_mesh_with_indexes(mesh, &mut face_indexes)
    }

    pub fn build_from_scene(scene: &Scene) -> BvhNode {
        let mut indexes: Vec<usize> = (0..scene.elements.len()).collect();
        BvhNode::build_from_scene_with_indexes(scene, &mut indexes)
    }

    pub fn intersect_for_mesh(&self, mesh: &Mesh, ray: &Ray, intersection: &mut Intersection) -> bool {
        if !self.aabb.intersect_ray(ray).0 {
            return false;
        }

        let mut any_hit = false;
        if self.children.is_empty() {
            // leaf node
            for face_index in &self.indexes {
                let face = &mesh.faces[*face_index];
                if intersect_polygon(&mesh.vertexes[face.v0], &mesh.vertexes[face.v1], &mesh.vertexes[face.v2], ray, intersection) {
                    any_hit = true;
                }
            }
        } else {
            // intermediate node
            for child in &self.children {
                if child.intersect_for_mesh(mesh, ray, intersection) {
                    any_hit = true;
                }
            }
        }

        any_hit
    }

    pub fn intersect_for_scene(&self, scene: &Scene, ray: &Ray, intersection: &mut Intersection) -> Option<usize> {
        if !self.aabb.intersect_ray(ray).0 {
            return None;
        }

        let mut nearest_index: Option<usize> = None;
        if self.children.is_empty() {
            // leaf node
            for index in &self.indexes {
                let e = &scene.elements[*index];
                if e.intersect(ray, intersection) {
                    nearest_index = Some(*index);
                }
            }
        } else {
            // intermediate node
            for child in &self.children {
                if let Some(index) = child.intersect_for_scene(scene, ray, intersection) {
                    nearest_index = Some(index);
                }
            }
        }

        nearest_index
    }
}

pub fn intersect_polygon(v0: &Vector3, v1: &Vector3, v2: &Vector3, ray: &Ray, intersection: &mut Intersection) -> bool {
    let ray_inv = -ray.direction;
    let edge1 = *v1 - *v0;
    let edge2 = *v2 - *v0;
    let denominator = det(&edge1, &edge2, &ray_inv);
    if denominator == 0.0 { return false; }

    let denominator_inv = denominator.recip();
    let d = ray.origin - *v0;

    let u = det(&d, &edge2, &ray_inv) * denominator_inv;
    if u < 0.0 || u > 1.0 { return false; }

    let v = det(&edge1, &d, &ray_inv) * denominator_inv;
    if v < 0.0 || u + v > 1.0 { return false; };

    let t = det(&edge1, &edge2, &d) * denominator_inv;
    if t < 0.0 || t > intersection.distance { return false; }

    intersection.position = ray.origin + ray.direction * t;
    intersection.normal = edge1.cross(&edge2).normalize();
    intersection.distance = t;
    intersection.uv = Vector2::new(u, v);
    true
}
