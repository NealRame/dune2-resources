use std::ops::{Add, Sub, Mul};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Mul<T> for Point where T: Mul<u32, Output = u32> + Copy {
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        Self {
            x: rhs*self.x,
            y: rhs*self.y,
        }
    }
}

impl Mul<Point> for u32 {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        return rhs*self;
    }
}
