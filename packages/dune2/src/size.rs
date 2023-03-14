#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn zero() -> Self {
        Self { width: 0, height: 0 }
    }
}
