use consts;
use vector::Vector3;
use texture::Texture;

pub enum SurfaceType {
    Diffuse,
    Specular,
    Refraction { refractive_index: f64 },
    GGX { roughness: f64 },
    GGXReflection { roughness: f64, refractive_index: f64 },
}

pub struct Material {
    pub surface: SurfaceType,
    pub albedo: Vector3,
    pub emission: Vector3,
    pub albedo_texture: Texture,
}

impl Material {
    pub fn new() -> Material {
        Material {
            surface: SurfaceType::Diffuse {},
            albedo: Vector3::from_one(1.0),
            emission: Vector3::from_one(1.0),
            albedo_texture: Texture::new(consts::WHITE_TEXTURE_PATH),
        }
    }
}
