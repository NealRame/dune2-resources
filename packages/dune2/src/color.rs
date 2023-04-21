use std::collections::HashMap;

use std::fs;
use std::io;
use std::path;

use std::error::{Error};

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

    pub fn from_pal_file(
        path: &path::PathBuf,
    ) -> Result<Self, Box<dyn Error>> {
        let mut reader = fs::File::open(path)?;
        Self::from_pal_reader(&mut reader)
    }

    pub fn from_pal_reader(
        reader: &mut impl io::Read,
    ) -> Result<Palette, Box<dyn Error>> {
        let mut palette = Palette::new();
        let mut buf = [0; 3];

        loop {
            let color = match reader.read(&mut buf)? {
                0 => break,
                3 => Color::new(4*buf[0], 4*buf[1], 4*buf[2]),
                _ => return Err(format!("Invalid palette file").into()),
            };
            palette.push(&color);
        }

        Ok(palette)
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

    pub fn color_at(&self, index: usize) -> Color {
        self.colors[index]
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }
}
