use vector::Vector3;

#[derive(Clone, Debug)]
pub enum SurfaceType {
    Diffuse,
    Specular,
    Reflection { refractiveIndex: f64 },
    GGX { roughness: f64 },
    GGXReflection { roughness: f64, refractiveIndex: f64 },
}

#[derive(Clone, Debug)]
pub struct Material {
    pub albedo: Vector3,
    pub emission: Vector3,
    pub surface: SurfaceType,
}

impl Material {
    pub fn new() -> Material {
        Material {
            albedo: Vector3::from_one(1.0),
            emission: Vector3::zero(),
            surface: SurfaceType::Diffuse {}
        }
    }
}