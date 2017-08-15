extern crate rand;

use self::rand::{Rng, ThreadRng};

use vector::{Vector3, Vector2};

#[derive(Debug)]
pub struct Camera {
    pub eye: Vector3,
    pub forward: Vector3,

    // レンズの半径
    pub lens_radius: f64,

    // 焦点距離
    pub focus_distance: f64,

    // 焦点面の基底ベクトル（正規化されている）
    pub right: Vector3,
    pub up: Vector3,

    // 焦点面の基底ベクトル（スクリーンの大きさが掛けられている）
    pub plane_half_right: Vector3,
    pub plane_half_up: Vector3,
}

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

impl Camera {
    pub fn new(eye: Vector3, target: Vector3, y_up: Vector3, v_fov: f64, aperture: f64, focus_distance: f64) -> Camera {
        let lens_radius = 0.5 * aperture;
        let plane_half_height = v_fov.to_radians().tan();
        let forward = (target - eye).normalize();
        let right = forward.cross(&y_up).normalize();
        let up = right.cross(&forward).normalize();

        Camera {
            eye: eye,
            lens_radius: lens_radius,
            focus_distance: focus_distance,
            forward: forward,
            right: right,
            up: up,
            plane_half_right: right * plane_half_height * focus_distance,
            plane_half_up: up * plane_half_height * focus_distance,
        }
    }

    fn sample_square(mut rng: &mut ThreadRng) -> Vector2 {
        let (u, v) = rng.gen::<(f64, f64)>();
        Vector2::new(u + 0.5, v + 0.5)
    }

    pub fn ray_with_dof(&self, normalized_coord: &Vector2, rng: &mut ThreadRng) -> Ray {
        let lens_uv = Camera::sample_square(rng) * self.lens_radius;
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

/*pub struct CameraBuilder {
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
}*/