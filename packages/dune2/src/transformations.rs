use std::ops::{Mul};

use crate::point::*;

#[derive(Debug, Clone, Copy,)]
pub struct TransformMatrix (
    (f32, f32, f32, ),
    (f32, f32, f32, ),
    (f32, f32, f32, ),
);

impl Mul<TransformMatrix> for TransformMatrix {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        // self coef
        let (a11, a12, a13) = self.0;
        let (a21, a22, a23) = self.1;
        let (a31, a32, a33) = self.2;

        // rhs coef
        let (b11, b12, b13) = rhs.0;
        let (b21, b22, b23) = rhs.1;
        let (b31, b32, b33) = rhs.2;

        Self (
            (
                a11*b11 + a12*b21 + a13*b31,
                a11*b12 + a12*b22 + a13*b32,
                a11*b13 + a12*b23 + a13*b33,
            ),
            (
                a21*b11 + a22*b21 + a23*b31,
                a21*b12 + a22*b22 + a23*b32,
                a21*b13 + a22*b23 + a23*b33,
            ),
            (
                a31*b11 + a32*b21 + a33*b31,
                a31*b12 + a32*b22 + a33*b32,
                a31*b13 + a32*b23 + a33*b33,
            ),
        )
    }
}

impl Mul<Point> for TransformMatrix {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        let (a11, a12, a13) = self.0;
        let (a21, a22, a23) = self.1;
        let (a31, a32, a33) = self.2;

        let x = rhs.x as f32;
        let y = rhs.y as f32;

        let a = a11*x + (y*a12 + a13);
        let b = a21*x + (y*a22 + a23);
        let c = a31*x + (y*a32 + a33);

        Point {
            x: (a/c) as u32,
            y: (b/c) as u32,
        }
    }
}

impl TransformMatrix {
    pub fn identity() -> Self {
        Self (
            (1.0, 0.0, 0.0, ),
            (0.0, 1.0, 0.0, ),
            (0.0, 0.0, 1.0, ),
        )
    }

    pub fn translate(x: f32, y: f32) -> Self {
        Self (
            (1.0, 0.0, x, ),
            (0.0, 1.0, y, ),
            (0.0, 0.0, 1.0, ),
        )
    }

    pub fn rotate(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self (
            (cos, -sin, 0.0, ),
            (sin, cos, 0.0, ),
            (0.0, 0.0, 1.0, ),
        )
    }
}
