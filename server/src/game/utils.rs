use rand::Rng;
use super::common::*;

pub fn generate_random_point<R: Rng>(
    rng: &mut R,
    x_range: (f64, f64),
    y_range: (f64, f64),
    thres: f64,
    pts: &[Point],
) -> Point {
    loop {
        let point = Point {
            x: rng.gen_range(x_range.0, x_range.1),
            y: rng.gen_range(y_range.0, y_range.1),
        };

        if pts.iter().all(|p| (point - *p).abs() >= thres) {
            return point;
        }
    }
}
