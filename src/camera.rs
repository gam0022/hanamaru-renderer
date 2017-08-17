extern crate rand;

use self::rand::{Rng, ThreadRng};

use vector::{Vector3, Vector2};

#[derive(Debug)]
pub struct Camera {
    // カメラの位置
    pub eye: Vector3,

    // レンズ形状
    pub lens_shape: LensShape,

    // レンズの半径
    pub lens_radius: f64,

    // 焦点距離
    pub focus_distance: f64,

    // 焦点面の基底ベクトル（正規化されている）
    pub right: Vector3,
    pub up: Vector3,
    pub forward: Vector3,

    // 焦点面の基底ベクトル（スクリーンの半分の大きさを乗算済み）
    pub plane_half_right: Vector3,
    pub plane_half_up: Vector3,
}

#[derive(Debug)]
pub enum LensShape {
    Square,
    Circle,
}

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

impl Camera {
    pub fn new(eye: Vector3, target: Vector3, y_up: Vector3, v_fov: f64,
               lens_shape: LensShape, aperture: f64, focus_distance: f64) -> Camera {
        let lens_radius = 0.5 * aperture;
        let plane_half_height = v_fov.to_radians().tan();
        let forward = (target - eye).normalize();
        let right = forward.cross(&y_up).normalize();
        let up = right.cross(&forward).normalize();

        Camera {
            eye: eye,
            lens_shape: lens_shape,
            lens_radius: lens_radius,
            focus_distance: focus_distance,
            forward: forward,
            right: right,
            up: up,
            plane_half_right: right * plane_half_height * focus_distance,
            plane_half_up: up * plane_half_height * focus_distance,
        }
    }

    fn sample_on_lens(&self, mut rng: &mut ThreadRng) -> Vector2 {
        loop {
            let (u, v) = rng.gen::<(f64, f64)>();
            let square = Vector2::new(2.0 * u - 1.0, 2.0 * v - 1.0);
            match self.lens_shape {
                LensShape::Square => {
                    return square;
                },
                LensShape::Circle => {
                    if square.norm() < 1.0 {
                        return square;
                    }
                }
            }
        }
    }

    pub fn ray_with_dof(&self, normalized_coord: &Vector2, rng: &mut ThreadRng) -> Ray {
        let lens_uv = self.sample_on_lens(rng) * self.lens_radius;
        let lens_pos = self.right * lens_uv.x + self.up * lens_uv.y;

        Ray {
            origin: self.eye + lens_pos,
            direction: (
                normalized_coord.x * self.plane_half_right
                    + normalized_coord.y * self.plane_half_up
                    + self.focus_distance * self.forward
                    - lens_pos
            ).normalize(),
        }
    }

    pub fn ray(&self, normalized_coord: &Vector2) -> Ray {
        Ray {
            origin: self.eye,
            direction: (
                normalized_coord.x * self.plane_half_right
                    + normalized_coord.y * self.plane_half_up
                    + self.focus_distance * self.forward
            ).normalize(),
        }
    }
}
