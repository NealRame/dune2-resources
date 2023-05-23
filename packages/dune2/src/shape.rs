use std::ops::{Mul};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Shape {
    pub rows: u32,
    pub columns: u32,
}

impl Mul<u32> for Shape {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self {
        Self {
            columns: rhs*self.columns,
            rows: rhs*self.rows,
        }
    }
}

impl Mul<Shape> for u32 {
    type Output = Shape;
    fn mul(self, rhs: Shape) -> Shape {
        return rhs*self;
    }
}

macro_rules! generate_uint_mul_impl {
    ($($t:ty),*) => {
        $(
            impl Mul<$t> for Shape {
                type Output = Self;
                fn mul(self, rhs: $t) -> Self {
                    return self*(rhs as u32);
                }
            }

            impl Mul<Shape> for $t {
                type Output = Shape;
                fn mul(self, rhs: Shape) -> Shape {
                    return rhs*self;
                }
            }
        )*
    };
}

generate_uint_mul_impl!(u8, u16);
