use texture::Texture;
use color::Color;
use vector::Vector3;
use camera::Ray;
use config;
use math::saturate;

#[derive(Clone, Debug)]
pub enum SurfaceType {
    Diffuse,
    Specular,
    Refraction { refractive_index: f64 },
    GGX { f0: f64 },
    GGXRefraction { refractive_index: f64 },
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
            SurfaceType::GGX { f0: _ } => true,

            SurfaceType::Specular => false,
            SurfaceType::Refraction { refractive_index: _ } => false,
            SurfaceType::GGXRefraction { refractive_index: _ } => false,
        }
    }

    pub fn bsdf(&self, view: &Vector3, normal: &Vector3, light: &Vector3) -> f64 {
        match self.surface {
            SurfaceType::Diffuse => config::PI.recip(),
            SurfaceType::Specular => unimplemented!(),
            SurfaceType::Refraction { refractive_index: _ } => unimplemented!(),
            SurfaceType::GGX { f0 } => {
                // https://schuttejoe.github.io/post/ggximportancesamplingpart1/
                // i: view, g: light, m: half

                // https://qiita.com/_Pheema_/items/f1ffb2e38cc766e6e668
                let alpha2 = roughness_to_alpha2(self.roughness);
                let half = (*light + *view).normalize();

                let l_dot_n = light.dot(normal);
                if l_dot_n.is_sign_negative() {
                    return 0.0;
                }

                let v_dot_n = view.dot(normal);
                let v_dot_h = view.dot(&half);
                let h_dot_n = half.dot(normal);

                // D: Microfacet Distribution Functions GGX(Trowbridge-Reitz model)
                let tmp = 1.0 - (1.0 - alpha2) * h_dot_n * h_dot_n;
                let d = alpha2 / (config::PI * tmp * tmp);

                // G: Masking-Shadowing Fucntion
                let g = g_smith_joint(l_dot_n, v_dot_n, alpha2);

                // F: Fresnel term
                let f = f_schlick_f64(v_dot_h, f0);

                d * g * f / (4.0 * l_dot_n * v_dot_n)
            }
            SurfaceType::GGXRefraction { refractive_index: _ } => unimplemented!()
        }
    }

    pub fn sample(&self, random: (f64, f64), position: &Vector3, view: &Vector3, normal: &Vector3) -> Option<SampleResult> {
        let ray = -*view;

        match self.surface {
            SurfaceType::Diffuse => {
                Some(SampleResult {
                    ray: Ray {
                        origin: *position + *normal * config::OFFSET,
                        direction: importance_sample_diffuse(random, normal),
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
                sample_refraction(random, position, &ray, normal, refractive_index)
            }
            SurfaceType::GGX { f0 } => {
                let alpha2 = roughness_to_alpha2(self.roughness);
                let half = importance_sample_ggx_half(random, normal, alpha2);
                let next_direction = ray.reflect(&half);

                // 半球外が選ばれた場合はBRDFを0にする
                let l_dot_n = next_direction.dot(normal);
                if l_dot_n.is_sign_negative() {
                    None
                } else {
                    let v_dot_n = view.dot(normal);
                    let v_dot_h = view.dot(&half);
                    let h_dot_n = half.dot(normal);

                    // G: Masking-Shadowing Fucntion
                    let g = g_smith_joint(l_dot_n, v_dot_n, alpha2);

                    // F: Fresnel term
                    let f = f_schlick_f64(v_dot_h, f0);

                    Some(SampleResult {
                        ray: Ray {
                            origin: *position + *normal * config::OFFSET,
                            direction: next_direction,
                        },
                        reflectance: f * saturate(g * v_dot_h / (h_dot_n * v_dot_n)),
                    })
                }
            }
            SurfaceType::GGXRefraction { refractive_index } => {
                let alpha2 = roughness_to_alpha2(self.roughness);
                let half = importance_sample_ggx_half(random, normal, alpha2);
                sample_refraction(random, position, &ray, &half, refractive_index)
            }
        }
    }
}

fn sample_refraction(random: (f64, f64), position: &Vector3, view: &Vector3, normal: &Vector3, refractive_index: f64) -> Option<SampleResult> {
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
        // 入射角をi、屈折角をt、r_sをS波の反射率、r_pをP波の反射率、frをフレネル反射率とする
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

// 法線を基準とした空間の基底ベクトルを計算
#[allow(dead_code)]
fn get_tangent_space_basis_gram_schmidtd(normal: &Vector3) -> (Vector3, Vector3) {
    let up = if normal.x.abs() > config::EPS {
        Vector3::new(0.0, 1.0, 0.0)
    } else {
        Vector3::new(1.0, 0.0, 0.0)
    };
    let tangent = up.cross(&normal).normalize();
    let binormal = normal.cross(&tangent);// up,tangent は直交かつ正規化されているので、normalize 不要
    (tangent, binormal)
}

// Duff et al.,の手法
// https://shikihuiku.wordpress.com/2018/07/09/%E6%AD%A3%E8%A6%8F%E7%9B%B4%E4%BA%A4%E5%9F%BA%E5%BA%95%E3%81%AE%E4%BD%9C%E3%82%8A%E6%96%B9%E3%81%AB%E3%81%A4%E3%81%84%E3%81%A6%E3%80%81%E6%94%B9%E3%82%81%E3%81%A6%E5%8B%89%E5%BC%B7%E3%81%97%E3%81%BE/
fn get_tangent_space_basis_revised_onb(normal: &Vector3) -> (Vector3, Vector3) {
    let s = normal.z.signum();
    let a = -(s + normal.z).recip();
    let b = normal.x * normal.y * a;
    let tangent = Vector3::new(1.0 + s * normal.x * normal.x * a, s * b, -s * normal.x);
    let binormal = Vector3::new(b, s + normal.y * normal.y * a, -normal.y);
    (tangent, binormal)
}

// 完全拡散反射のcos項による重点サンプリング
// https://github.com/githole/edupt/blob/master/radiance.h
fn importance_sample_diffuse(random: (f64, f64), normal: &Vector3) -> Vector3 {
    let (tangent, binormal) = get_tangent_space_basis_revised_onb(normal);

    // θ,φは極座標系の偏角。cosθにより重点サンプリングをしたい
    // 任意の確率密度関数fを積分した累積分布関数Fの逆関数を一様乱数に噛ませれば、
    // 任意の確率密度を持つ確率変数を得ることができる（逆関数法）
    // ・f(θ,φ) = cos(θ)/PI
    // ・F(θ,φ) = ∬f(θ,φ) dθdφ = φ/2PI * (1 - (cosθ)^2)
    // ・F(θ) = 1 - (cosθ)^2
    // ・F(φ) = φ/2PI
    // Fの逆関数から、角度θ,φを求めることができるので、
    //float theta = asin(sqrt(random.1));// θは整理すると消去できるのでコメントアウト
    let phi = config::PI2 * random.0;
    // サンプリング方向 result は極座標から直交座標への変換によって求められる
    // result = tangent * sin(theta) * cos(phi) + binormal * sin(theta) * sin(phi) + normal * cos(theta))
    // ここで、sin(theta)とcos(theta)は次のように整理できる
    // ・sin(theta) = sin(asin(sqrt(random.1))) = sqrt(random.1) = sqrt(random.1)
    // ・cos(theta) = sqrt(1.0 - sin(theta) * sin(theta)) = sqrt(1.0 - random.1)
    // よって、result = (tangent * cos(phi) + binormal* sin(phi)) * sin(theta) + normal * cos(theta))
    //               = (tangent * cos(phi) + binormal* sin(phi)) * sqrt(random.1) + normal * sqrt(1.0 - random.1)
    (tangent * phi.cos() + binormal * phi.sin()) * random.1.sqrt() + *normal * (1.0 - random.1).sqrt()
}

fn roughness_to_alpha2(roughness: f64) -> f64 {
    // UE4の結果に近づけたいなら、alpha = roughness にする
    // alpha = roughness * roughness の実装をよく見かける
    let alpha = roughness;
    alpha * alpha
}

// Unreal Engine 4 で利用されている ImportanceSampleGGX を移植
// cos項による重点サンプリングのためのハーフベクトルを計算
// http://project-asura.com/blog/?p=3124
fn importance_sample_ggx_half(random: (f64, f64), normal: &Vector3, alpha2: f64) -> Vector3 {
    let (tangent, binormal) = get_tangent_space_basis_revised_onb(normal);

    let phi = config::PI2 * random.0;
    let cos_theta = ((1.0 - random.1) / (1.0 + (alpha2 - 1.0) * random.1)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    let h = Vector3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
    tangent * h.x + binormal * h.y + *normal * h.z
}

fn g_smith_joint_lambda(x_dot_n: f64, alpha2: f64) -> f64 {
    let a = (x_dot_n * x_dot_n).recip() - 1.0;
    0.5 * (1.0 + alpha2 * a).sqrt() - 0.5
}

fn g_smith_joint(l_dot_n: f64, v_dot_n: f64, alpha2: f64) -> f64 {
    let lambda_l = g_smith_joint_lambda(l_dot_n, alpha2);
    let lambda_v = g_smith_joint_lambda(v_dot_n, alpha2);
    (1.0 + lambda_l + lambda_v).recip()
}

fn f_schlick_f64(v_dot_h: f64, f0: f64) -> f64 {
    f0 + (1.0 - f0) * (1.0 - v_dot_h).powi(5)
}
