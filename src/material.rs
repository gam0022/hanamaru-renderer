use consts;
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

#[derive(Clone, Debug)]
pub struct Material<'a> {
    pub surface: SurfaceType,
    pub albedo: Vector3,
    pub emission: Vector3,
    pub albedo_texture: &'a Texture,
}

impl<'a> Material<'a> {
    pub fn new() -> Material<'a> {
        Material {
            surface: SurfaceType::Diffuse {},
            albedo: Vector3::from_one(1.0),
            emission: Vector3::from_one(1.0),
            albedo_texture: &Texture::new(consts::WHITE_TEXTURE_PATH),
        }
    }
}
