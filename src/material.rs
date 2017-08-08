use vector::Vector3;
use texture::Texture;

#[derive(Clone, Debug)]
pub enum SurfaceType {
    Diffuse,
    Specular,
    Refraction { refractive_index: f64 },
    GGX { roughness: f64 },
    GGXReflection { roughness: f64, refractive_index: f64 },
}

#[derive(Debug)]
pub struct Material {
    pub surface: SurfaceType,
    pub albedo: Vector3,
    pub emission: Vector3,
    pub albedo_texture: Texture,
}

#[derive(Clone, Debug)]
pub struct PointMaterial {
    pub surface: SurfaceType,
    pub albedo: Vector3,
    pub emission: Vector3,
}

impl PointMaterial {
    pub fn new() -> PointMaterial {
        PointMaterial {
            surface: SurfaceType::Diffuse,
            albedo: Vector3::one(),
            emission: Vector3::zero(),
        }
    }
}
