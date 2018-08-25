use config;
use vector::Vector3;
use scene::Intersection;
use camera::Ray;
use color::Color;
use math::saturate;

pub fn diffuse_brdf() -> f64 {
    config::PI.recip()
}

pub fn ggx_brdf(view: &Vector3, light: &Vector3, normal: &Vector3, alpha2: f64, f0: f64) -> f64 {
    let half = (*light + *view).normalize();

    let v_dot_n = saturate(view.dot(normal));
    let l_dot_n = saturate(light.dot(normal));
    let v_dot_h = saturate(view.dot(&half));
    let h_dot_n = saturate(half.dot(normal));

    // Masking-shadowing関数
    let g = g_smith_joint(l_dot_n, v_dot_n, alpha2);

    // albedoをフレネル反射率のパラメータのF0として扱う
    let f = f_schlick_f64(v_dot_h, f0);

    f * saturate(g * v_dot_h / (4.0 * h_dot_n * v_dot_n))
}

// 法線を基準とした空間の基底ベクトルを計算
fn get_tangent_space_basis(normal: &Vector3) -> (Vector3, Vector3) {
    let up = if normal.x.abs() > config::EPS {
        Vector3::new(0.0, 1.0, 0.0)
    } else {
        Vector3::new(1.0, 0.0, 0.0)
    };
    let tangent = up.cross(&normal).normalize();
    let binormal = normal.cross(&tangent);// up,tangent は直交かつ正規化されているので、normalize 不要
    (tangent, binormal)
}

// 完全拡散反射のcos項による重点サンプリング
// https://github.com/githole/edupt/blob/master/radiance.h
pub fn importance_sample_diffuse(random: (f64, f64), normal: &Vector3) -> Vector3 {
    let (tangent, binormal) = get_tangent_space_basis(normal);

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

pub fn roughness_to_alpha2(roughness: f64) -> f64 {
    // UE4の結果に近づけたいなら、alpha = roughness にする
    // alpha = roughness * roughness の実装をよく見かける
    let alpha = roughness;
    alpha * alpha
}

// Unreal Engine 4 で利用されている ImportanceSampleGGX を移植
// cos項による重点サンプリングのためのハーフベクトルを計算
// http://project-asura.com/blog/?p=3124
pub fn importance_sample_ggx(random: (f64, f64), normal: &Vector3, alpha2: f64) -> Vector3 {
    let (tangent, binormal) = get_tangent_space_basis(normal);

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

pub fn g_smith_joint(l_dot_n :f64, v_dot_n: f64, alpha2: f64) -> f64 {
    let lambda_l = g_smith_joint_lambda(l_dot_n, alpha2);
    let lambda_v = g_smith_joint_lambda(v_dot_n, alpha2);
    (1.0 + lambda_l + lambda_v).recip()
}

pub fn f_schlick_f64(v_dot_h: f64, f0: f64) -> f64 {
    f0 + (1.0 - f0) * (1.0 - v_dot_h).powi(5)
}

pub fn f_schlick(v_dot_h: f64, f0: &Color) -> Color {
    Color::new(
        f_schlick_f64(v_dot_h,f0.x),
        f_schlick_f64(v_dot_h,f0.y),
        f_schlick_f64(v_dot_h,f0.z),
    )
}

pub fn sample_refraction(random: (f64, f64), normal: &Vector3, refractive_index: f64, intersection: &mut Intersection, ray: &mut Ray) {
    let is_incoming = ray.direction.dot(&normal).is_sign_negative();
    let oriented_normal = if is_incoming { *normal } else { -*normal };
    let nnt = if is_incoming { 1.0 / refractive_index } else { refractive_index };
    let reflect_direction = ray.direction.reflect(&oriented_normal);
    let refract_direction = ray.direction.refract(&oriented_normal, nnt);
    if refract_direction == Vector3::zero() {
        // 完全反射のケース
        ray.origin = intersection.position + config::OFFSET * oriented_normal;
        ray.direction = reflect_direction;
    } else {
        // フレネル反射率rの計算
        // 入射角をI、屈折角をT、r_sをS波の反射率、r_pをP波の反射率、rをフレネル反射率とする
        let cos_i = ray.direction.dot(&-oriented_normal);
        //float cos_t = sqrt(1.0 - nnt * nnt * (1.0 - cos_i * cos_i));
        let cos_t = refract_direction.dot(&-oriented_normal);
        let r_s = (nnt * cos_i - cos_t) * (nnt * cos_i - cos_t) / ((nnt * cos_i + cos_t) * (nnt * cos_i + cos_t));
        let r_p = (nnt * cos_t - cos_i) * (nnt * cos_t - cos_i) / ((nnt * cos_t + cos_i) * (nnt * cos_t + cos_i));
        let r = 0.5 * (r_s + r_p);
        if random.0 <= r {
            // 反射
            ray.origin = intersection.position + config::OFFSET * oriented_normal;
            ray.direction = reflect_direction;
        } else {
            // 屈折

            // 立体角の変化に伴う放射輝度の補正
            intersection.material.albedo *= nnt.powf(2.0);

            // 物体内部にレイの原点を移動する
            ray.origin = intersection.position - config::OFFSET * oriented_normal;
            ray.direction = refract_direction;
        }
    }
}
