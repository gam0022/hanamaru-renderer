use texture::Texture;
use color::Color;
use vector::Vector3;
use camera::Ray;
use config;
use bsdf;
use math::saturate;

#[derive(Clone, Debug)]
pub enum SurfaceType {
    Diffuse,
    Specular,
    Refraction { refractive_index: f64 },
    GGX { f0: f64 },
    GGXRefraction { f0: f64, refractive_index: f64 },
}

#[derive(Debug)]
pub struct Material {
    pub surface: SurfaceType,
    pub albedo: Texture,
    pub emission: Texture,
    pub roughness: Texture,
}

#[derive(Clone, Debug)]
pub struct PointMaterial {
    pub surface: SurfaceType,
    pub albedo: Color,
    pub emission: Color,
    pub roughness: f64,
}

pub struct SampleResult {
    pub ray: Ray,

    // reflectance = bsdf * cos(normal, light) / pdf
    // 重点的サンプリングを行うと、bsdf * cos を pdf が打ち消すケースが多いので、このような定義とした
    pub reflectance: f64,
}

impl PointMaterial {
    pub fn nee_available(&self) -> bool {
        match self.surface {
            SurfaceType::Diffuse => true,
            SurfaceType::GGX { f0 } => true,

            SurfaceType::Specular => false,
            SurfaceType::Refraction { refractive_index } => false,
            SurfaceType::GGXRefraction { f0, refractive_index } => false,
        }
    }

    pub fn bsdf(&self, view: &Vector3, normal: &Vector3, light: &Vector3) -> f64 {
        match self.surface {
            SurfaceType::Diffuse => config::PI.recip(),
            SurfaceType::Specular => unimplemented!(),
            SurfaceType::Refraction { refractive_index } => unimplemented!(),
            SurfaceType::GGX { f0 } => {
                // https://schuttejoe.github.io/post/ggximportancesamplingpart1/
                // i: view, g: light, m: half

                // https://qiita.com/_Pheema_/items/f1ffb2e38cc766e6e668
                let alpha2 = bsdf::roughness_to_alpha2(self.roughness);
                let half = (*light + *view).normalize();

                let l_dot_n = light.dot(normal);
                if l_dot_n.is_sign_negative() {
                    return 0.0;
                }

                let v_dot_n = view.dot(normal);
                let v_dot_h = view.dot(&half);
                let h_dot_n = half.dot(normal);

                // D: Microfacet Distribution Functions GGX(Trowbridge-Reitz model)
                let tmp = (1.0 - (1.0 - alpha2) * h_dot_n * h_dot_n);
                let d = alpha2 / (config::PI * tmp * tmp);

                // G: Masking-Shadowing Fucntion
                let g = bsdf::g_smith_joint(l_dot_n, v_dot_n, alpha2);

                // F: Fresnel term
                let f = bsdf::f_schlick_f64(v_dot_h, f0);

                d * g * f / (4.0 * l_dot_n * v_dot_n)
            }
            SurfaceType::GGXRefraction { f0, refractive_index } => unimplemented!()
        }
    }

    pub fn sample(&self, random: (f64, f64), position: &Vector3, view: &Vector3, normal: &Vector3) -> Option<SampleResult> {
        let ray = -*view;

        match self.surface {
            SurfaceType::Diffuse => {
                Some(SampleResult {
                    ray: Ray {
                        origin: *position + *normal * config::OFFSET,
                        direction: bsdf::importance_sample_diffuse(random, normal),
                    },
                    reflectance: 1.0,// bsdf * cos と pdf が打ち消し合う
                })
            }
            SurfaceType::Specular => {
                Some(SampleResult {
                    ray: Ray {
                        origin: *position + *normal * config::OFFSET,
                        direction: ray.reflect(normal),
                    },
                    reflectance: 1.0,// bsdf * cos と pdf が打ち消し合う
                })
            }
            SurfaceType::Refraction { refractive_index } => {
                self.sample_refraction(random, position, &ray, normal, refractive_index)
            }
            SurfaceType::GGX { f0 } => {
                let alpha2 = bsdf::roughness_to_alpha2(self.roughness);
                let half = bsdf::importance_sample_ggx(random, normal, alpha2);
                let next_direction = ray.reflect(&half);

                // 半球外が選ばれた場合はBRDFを0にする
                // 真値よりも暗くなるので、サンプリングやり直す方が理想的ではありそう
                let l_dot_n = next_direction.dot(normal);
                if l_dot_n.is_sign_negative() {
                    None
                } else {
                    let v_dot_n = view.dot(normal);
                    let v_dot_h = view.dot(&half);
                    let h_dot_n = half.dot(normal);

                    // G: Masking-Shadowing Fucntion
                    let g = bsdf::g_smith_joint(l_dot_n, v_dot_n, alpha2);

                    // F: Fresnel term
                    let f = bsdf::f_schlick_f64(v_dot_h, f0);

                    Some(SampleResult {
                        ray: Ray {
                            origin: *position + *normal * config::OFFSET,
                            direction: next_direction,
                        },
                        reflectance: f * saturate(g * v_dot_h / (h_dot_n * v_dot_n)),
                    })
                }
            }
            SurfaceType::GGXRefraction { f0, refractive_index } => {
                let alpha2 = bsdf::roughness_to_alpha2(self.roughness);
                let half = bsdf::importance_sample_ggx(random, normal, alpha2);
                self.sample_refraction(random, position, &ray, &half, refractive_index)
            }
        }
    }

    fn sample_refraction(&self, random: (f64, f64), position: &Vector3, view: &Vector3, normal: &Vector3, refractive_index: f64) -> Option<SampleResult> {
        let is_incoming = view.dot(&normal).is_sign_negative();
        let oriented_normal = if is_incoming { *normal } else { -*normal };
        let nnt = if is_incoming { refractive_index.recip() } else { refractive_index };
        let reflect_direction = view.reflect(&oriented_normal);
        let refract_direction = view.refract(&oriented_normal, nnt);
        if refract_direction == Vector3::zero() {
            // 全反射のケース
            Some(SampleResult {
                ray: Ray {
                    origin: *position + config::OFFSET * oriented_normal,
                    direction: reflect_direction,
                },
                reflectance: 1.0,// bsdf * cos と pdf が打ち消し合う
            })
        } else {
            // フレネル反射率rの計算
            // 入射角をI、屈折角をT、r_sをS波の反射率、r_pをP波の反射率、rをフレネル反射率とする
            let cos_i = view.dot(&-oriented_normal);
            //float cos_t = sqrt(1.0 - nnt * nnt * (1.0 - cos_i * cos_i));
            let cos_t = refract_direction.dot(&-oriented_normal);
            let r_s = (nnt * cos_i - cos_t) * (nnt * cos_i - cos_t) / ((nnt * cos_i + cos_t) * (nnt * cos_i + cos_t));
            let r_p = (nnt * cos_t - cos_i) * (nnt * cos_t - cos_i) / ((nnt * cos_t + cos_i) * (nnt * cos_t + cos_i));
            let fr = 0.5 * (r_s + r_p);

            if random.0 <= fr {
                // 反射
                Some(SampleResult {
                    ray: Ray {
                        origin: *position + config::OFFSET * oriented_normal,
                        direction: reflect_direction,
                    },
                    reflectance: 1.0,// bsdf * cos と pdf が打ち消し合う
                })
            } else {
                // 屈折
                Some(SampleResult {
                    ray: Ray {
                        origin: *position - config::OFFSET * oriented_normal,// 物体内部にレイの原点を移動する
                        direction: refract_direction,
                    },
                    reflectance: nnt * nnt,// 立体角の変化に伴う放射輝度の補正
                })
            }
        }
    }
}
