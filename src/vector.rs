use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, MulAssign};
use std::cmp::PartialEq;
use config;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 { x: x, y: y, z: z }
    }

    pub fn zero() -> Vector3 {
        Vector3::from_one(0.0)
    }

    pub fn one() -> Vector3 {
        Vector3::from_one(1.0)
    }

    pub fn from_one(v: f64) -> Vector3 {
        Vector3 { x: v, y: v, z: v }
    }

    pub fn length(&self) -> f64 {
        self.norm().sqrt()
    }

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub fn normalize(&self) -> Vector3 {
        let inv_len = self.length().recip();
        Vector3 {
            x: self.x * inv_len,
            y: self.y * inv_len,
            z: self.z * inv_len,
        }
    }

    pub fn dot(&self, other: &Vector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn reflect(&self, normal: &Vector3) -> Vector3 {
        *self - 2.0 * self.dot(&normal) * *normal
    }

    pub fn refract(&self, normal: &Vector3, refractive_index: f64) -> Vector3 {
        let k = 1.0 - refractive_index * refractive_index * (1.0 - normal.dot(self) * self.dot(normal));
        if k < 0.0 {
            Vector3::zero()
        } else {
            refractive_index * *self - (refractive_index * self.dot(normal) + k.sqrt()) * *normal
        }
    }

    pub fn xy(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    pub fn zy(&self) -> Vector2 {
        Vector2::new(self.z, self.y)
    }

    pub fn xz(&self) -> Vector2 {
        Vector2::new(self.x, self.z)
    }

    pub fn approximately(&self, other: &Vector3) -> bool {
        (*self - *other).norm() < config::OFFSET * 4.0
    }
}

impl Add for Vector3 {
    type Output = Vector3;

    fn add(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<f64> for Vector3 {
    type Output = Vector3;

    fn add(self, other: f64) -> Vector3 {
        Vector3 {
            x: self.x + other,
            y: self.y + other,
            z: self.z + other,
        }
    }
}

impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Sub<f64> for Vector3 {
    type Output = Vector3;

    fn sub(self, other: f64) -> Vector3 {
        Vector3 {
            x: self.x - other,
            y: self.y - other,
            z: self.z - other,
        }
    }
}

impl Mul for Vector3 {
    type Output = Vector3;

    fn mul(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl Mul<f64> for Vector3 {
    type Output = Vector3;

    fn mul(self, other: f64) -> Vector3 {
        Vector3 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl Mul<Vector3> for f64 {
    type Output = Vector3;

    fn mul(self, other: Vector3) -> Vector3 {
        other * self
    }
}

impl Div for Vector3 {
    type Output = Vector3;

    fn div(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl Div<f64> for Vector3 {
    type Output = Vector3;

    fn div(self, other: f64) -> Vector3 {
        Vector3 {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl Neg for Vector3 {
    type Output = Vector3;

    fn neg(self) -> Vector3 {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl PartialEq for Vector3 {
    fn eq(&self, other: &Vector3) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, other: Vector3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl MulAssign for Vector3 {
    fn mul_assign(&mut self, other: Vector3) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
    }
}

impl MulAssign<f64> for Vector3 {
    fn mul_assign(&mut self, other: f64) {
        self.x *= other;
        self.y *= other;
        self.z *= other;
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Vector2 {
        Vector2 { x: x, y: y}
    }

    pub fn zero() -> Vector2 {
        Vector2::from_one(0.0)
    }

    pub fn from_one(v: f64) -> Vector2 {
        Vector2 { x: v, y: v }
    }

    pub fn length(&self) -> f64 {
        self.norm().sqrt()
    }

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y)
    }

    pub fn normalize(&self) -> Vector2 {
        let inv_len = self.length().recip();
        Vector2 {
            x: self.x * inv_len,
            y: self.y * inv_len,
        }
    }

    pub fn dot(&self, other: &Vector2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(&self, other: &Vector2) -> f64 {
        self.x * other.y - other.x * self.y
    }

    pub fn approximately(&self, other: &Vector2) -> bool {
        (*self - *other).norm() < config::OFFSET * 4.0
    }
}

impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<f64> for Vector2 {
    type Output = Vector2;

    fn add(self, other: f64) -> Vector2 {
        Vector2 {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub<f64> for Vector2 {
    type Output = Vector2;

    fn sub(self, other: f64) -> Vector2 {
        Vector2 {
            x: self.x - other,
            y: self.y - other,
        }
    }
}

impl Mul for Vector2 {
    type Output = Vector2;

    fn mul(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl Mul<f64> for Vector2 {
    type Output = Vector2;

    fn mul(self, other: f64) -> Vector2 {
        Vector2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Mul<Vector2> for f64 {
    type Output = Vector2;

    fn mul(self, other: Vector2) -> Vector2 {
        other * self
    }
}

impl Div for Vector2 {
    type Output = Vector2;

    fn div(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl Div<f64> for Vector2 {
    type Output = Vector2;

    fn div(self, other: f64) -> Vector2 {
        Vector2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Neg for Vector2 {
    type Output = Vector2;

    fn neg(self) -> Vector2 {
        Vector2 {
            x: -self.x,
            y: -self.y,
        }
    }
}
