use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
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

    pub fn add(&mut self, color: &Color) -> usize {
        match self.colors_index.get(color) {
            Some(index) => *index,
            None => {
                let index = self.colors.len();
                self.colors_index.insert(*color, index);
                self.colors.push(*color);
                index
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Color> + '_ {
        self.colors.iter().copied()
    }

    pub fn color_index(&self, color: &Color) -> Option<usize> {
        self.colors_index.get(&color).copied()
    }

    pub fn color_at(&self, index: usize) -> Color {
        self.colors[index]
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }
}
