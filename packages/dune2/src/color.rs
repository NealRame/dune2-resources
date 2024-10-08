use std::collections::HashMap;
use std::ops::Mul;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl<T> Mul<T> for Color where T: Mul<u8, Output = u8> + Copy {
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        Self {
            red: rhs*self.red,
            green: rhs*self.green,
            blue: rhs*self.blue,
        }
    }
}

impl Mul<Color> for u8 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color {
        return rhs*self;
    }
}

impl From<&[u8; 3]> for Color {
    fn from(rgb: &[u8; 3]) -> Self {
        Self {
            red: rgb[0],
            green: rgb[1],
            blue: rgb[2],
        }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((red, green, blue): (u8, u8, u8)) -> Self {
        Self {
            red,
            green,
            blue,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Palette {
    colors_index: HashMap<Color, usize>,
    colors: Vec<Color>,
}

impl Palette {
    pub fn new() -> Self {
        Self {
            colors_index: HashMap::new(),
            colors: Vec::new(),
        }
    }

    pub fn push(&mut self, color: &Color) -> usize {
        let index = self.colors.len();
        self.colors_index.insert(*color, index);
        self.colors.push(*color);
        index
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, Color)> + '_ {
        self.colors.iter().copied().enumerate()
    }

    pub fn color_index(&self, color: &Color) -> Option<usize> {
        self.colors_index.get(&color).copied()
    }

    pub fn color_at(&self, index: usize) -> Option<Color> {
        self.colors.get(index).copied()
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }
}
