use std::ops::Mul;
use std::fmt;

use serde::{Deserialize, Serialize};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::shape::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn zero() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}

impl Mul<u32> for Size {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self {
        Self {
            width: rhs*self.width,
            height: rhs*self.height,
        }
    }
}

impl Mul<Size> for u32 {
    type Output = Size;
    fn mul(self, rhs: Size) -> Size {
        return rhs*self;
    }
}

macro_rules! generate_uint_mul_impl {
    ($($t:ty),*) => {
        $(
            impl Mul<$t> for Size {
                type Output = Self;
                fn mul(self, rhs: $t) -> Self {
                    return self*(rhs as u32);
                }
            }

            impl Mul<Size> for $t {
                type Output = Size;
                fn mul(self, rhs: Size) -> Size {
                    return rhs*self;
                }
            }
        )*
    };
}

generate_uint_mul_impl!(u8, u16);

impl Mul<Shape> for Size {
    type Output = Self;
    fn mul(self, rhs: Shape) -> Self {
        return Self {
            width: self.width*rhs.columns,
            height: self.height*rhs.rows,
        };
    }
}

impl Mul<Size> for Shape {
    type Output = Size;
    fn mul(self, rhs: Size) -> Size {
        return rhs*self;
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}
