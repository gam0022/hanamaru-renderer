use vector::Vector3;

pub fn reinhard(color: &Vector3, exposure: f64, white_point: &Vector3) -> Vector3 {
    let color = *color * exposure;
    return (color / (color + 1.0)
        * (color / (*white_point * *white_point) + 1.0)
    ).saturate();
}
