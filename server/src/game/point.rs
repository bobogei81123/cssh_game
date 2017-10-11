use std::ops::*;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn from_angle(angle: f64) -> Self {
        Self {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    pub fn abs(&self) -> f64 {
        f64::hypot(self.x, self.y)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {x: self.x + other.x, y: self.y + other.y}
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {x: self.x - other.x, y: self.y - other.y}
    }
}

impl Mul for Point {
    type Output = f64;

    fn mul(self, other: Point) -> Self::Output {
        self.x * other.x + self.y * other.y
    }
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: x,
            y: y,
        }
    }
}
