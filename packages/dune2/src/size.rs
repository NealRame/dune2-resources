use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn scaled(&self, scale: u32) -> Self {
        Self {
            width: self.width*scale,
            height: self.height*scale,
        }
    }
}

impl Size {
    pub fn zero() -> Self {
        Self { width: 0, height: 0 }
    }
}
