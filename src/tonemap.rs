use vector::Vector3;

pub fn reinhard(color: &Vector3, exposure: f64) -> Vector3 {
    let color = *color * exposure;
    return (color / (Vector3::one() + color)).saturate();
}
