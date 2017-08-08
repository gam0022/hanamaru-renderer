use vector::Vector3;
use texture::Texture;
use color::Color;

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
    pub albedo: Texture,
    pub emission: Texture,
}

#[derive(Clone, Debug)]
pub struct PointMaterial {
    pub surface: SurfaceType,
    pub albedo: Color,
    pub emission: Color,
}

impl PointMaterial {
    pub fn new() -> PointMaterial {
        PointMaterial {
            surface: SurfaceType::Diffuse,
            albedo: Color::one(),
            emission: Color::zero(),
        }
    }
}
