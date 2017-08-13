extern crate rand;

use self::rand::{thread_rng, Rng, ThreadRng};

use vector::Vector3;
use scene::Mesh;
use consts;

pub struct BvhNode {
    pub left_bottom: Vector3,
    pub right_top: Vector3,

    // size must be 0 or 2
    // empty means leaf node
    pub children: Vec<Box<BvhNode>>,

    // has faces means leaf node
    pub face_indexes: Vec<usize>,
}

impl BvhNode {
    fn empty() -> BvhNode {
        BvhNode {
            left_bottom: Vector3::new(consts::INF, consts::INF, consts::INF),
            right_top: Vector3::new(-consts::INF, -consts::INF, -consts::INF),
            children: vec![],
            face_indexes: vec![],
        }
    }

    fn set_aabb(&mut self, mesh: &Mesh, face_indexes: &Vec<usize>) {
        for face_index in face_indexes {
            let face = &mesh.faces[*face_index];
            let v0 = mesh.vertexes[face.v0];
            let v1 = mesh.vertexes[face.v1];
            let v2 = mesh.vertexes[face.v2];

            self.left_bottom.x = self.left_bottom.x.min(v0.x).min(v1.x).min(v2.x);
            self.left_bottom.y = self.left_bottom.y.min(v0.y).min(v1.y).min(v2.y);
            self.left_bottom.z = self.left_bottom.z.min(v0.z).min(v1.z).min(v2.z);

            self.right_top.x = self.left_bottom.x.max(v0.x).max(v1.x).max(v2.x);
            self.right_top.y = self.left_bottom.y.max(v0.y).max(v1.y).max(v2.y);
            self.right_top.z = self.left_bottom.z.max(v0.z).max(v1.z).max(v2.z);
        }
    }

    fn from_face_indexes(mesh: &Mesh, face_indexes: &mut Vec<usize>, rng: &mut ThreadRng) -> BvhNode {
        let mut node = BvhNode::empty();
        node.set_aabb(mesh, face_indexes);

        let mid = face_indexes.len() / 2;
        if mid > 2 {
            let axis = rng.gen_range(0, 2);// 乱数で分割する軸を選ぶ
            match axis {
                // X
                0 => {
                    face_indexes.sort_by(|a, b| {
                        let a_face = &mesh.faces[*a];
                        let b_face = &mesh.faces[*b];
                        let a_sum = mesh.vertexes[a_face.v0].x + mesh.vertexes[a_face.v1].x + mesh.vertexes[a_face.v2].x;
                        let b_sum = mesh.vertexes[b_face.v0].x + mesh.vertexes[b_face.v1].x + mesh.vertexes[b_face.v2].x;
                        a_sum.partial_cmp(&b_sum).unwrap()
                    });
                }
                // Y
                1 => {
                    face_indexes.sort_by(|a, b| {
                        let a_face = &mesh.faces[*a];
                        let b_face = &mesh.faces[*b];
                        let a_sum = mesh.vertexes[a_face.v0].y + mesh.vertexes[a_face.v1].y + mesh.vertexes[a_face.v2].y;
                        let b_sum = mesh.vertexes[b_face.v0].y + mesh.vertexes[b_face.v1].y + mesh.vertexes[b_face.v2].y;
                        a_sum.partial_cmp(&b_sum).unwrap()
                    });
                }
                // Z
                _ => {
                    face_indexes.sort_by(|a, b| {
                        let a_face = &mesh.faces[*a];
                        let b_face = &mesh.faces[*b];
                        let a_sum = mesh.vertexes[a_face.v0].z + mesh.vertexes[a_face.v1].z + mesh.vertexes[a_face.v2].z;
                        let b_sum = mesh.vertexes[b_face.v0].z + mesh.vertexes[b_face.v1].z + mesh.vertexes[b_face.v2].z;
                        a_sum.partial_cmp(&b_sum).unwrap()
                    });
                }
            };

            let mut left_face_indexes = face_indexes.split_off(mid);
            node.children.push(Box::new(BvhNode::from_face_indexes(mesh, face_indexes, rng)));
            node.children.push(Box::new(BvhNode::from_face_indexes(mesh, &mut left_face_indexes, rng)));
        } else {
            node.face_indexes = face_indexes.clone();
        }

        node
    }

    pub fn from_mesh(mesh: &Mesh) -> BvhNode {
        let mut rng = thread_rng();
        let mut face_indexes: Vec<usize> = (0..mesh.faces.len()).collect();
        BvhNode::from_face_indexes(mesh, &mut face_indexes, &mut rng)
    }
}
