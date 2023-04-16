use std::error::{ Error };
use std::io::{Read, Seek };
use std::path::{ PathBuf };

use std::fs;

#[derive(Clone, Copy, Debug)]
pub struct Shape {
    rows: usize,
    columns: usize,
}

impl Shape {
    pub fn from_index(index: usize) -> Shape {
        match index {
            10 => {
                Shape { rows: 3, columns: 3 }
            },
            11|25 => {
                Shape { rows: 2, columns: 2 }
            },
            12|13 => {
                Shape { rows: 3, columns: 2 }
            },
            14..=18|24 => {
                Shape { rows: 2, columns: 2 }
            },
            19 => {
                Shape { rows: 3, columns: 3 }
            },
            20|21 => {
                Shape { rows: 3, columns: 2 }
            },
            _ => {
                Shape { rows: 1, columns: 1 }
            },
        }
    }
}

pub struct Icon {
    pub shape: Shape,
    pub tiles: Vec<usize>,
}

pub struct Map {
    pub icons: Vec<Icon>,
}

impl Map {
    pub fn from_reader<T>(
        reader: &mut T,
    ) -> Result<Map, Box<dyn Error>> where T: Read + Seek {
        let mut buf = [0u8; 2];
        let mut indexes = Vec::new();
        loop {
            match reader.read(&mut buf)? {
                0 => break,
                2 => indexes.push(u16::from_le_bytes(buf) as usize),
                _ => return Err("Unexpected end of file".into()),
            }
        }

        let count = *(indexes.first().unwrap());
        let icons =
            std::iter::zip(
                indexes.iter().skip(1).take(count - 1),
                indexes.iter().skip(2).take(count - 1),
            )
            .map(|(start, end)| {
                let start = *start as usize;
                let end = *end as usize;
                (start, if end == 0 { indexes.len() } else { end })
            })
            .enumerate()
            .flat_map(|(icon_index, (start, end))| {
                let count = end - start;

                let mut shape = Shape::from_index(icon_index);

                if shape.rows*shape.columns == 1 {
                    shape.columns = count;
                }

                let shape_size = shape.rows*shape.columns;
                let shape_count = count/shape_size;

                let mut icons = Vec::with_capacity(shape_count);
                for i in 0..shape_count {
                    let start = start + i*shape_size;
                    let end = start + shape_size;
                    let tiles = indexes[start..end].to_vec();
                    icons.push(Icon {
                        shape,
                        tiles,
                    })
                }

                icons
            })
            .collect::<Vec<_>>();

        Ok(Map { icons })
    }
}

impl std::convert::TryFrom<PathBuf> for Map {
    type Error = Box<dyn Error>;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let mut reader = fs::File::open(path)?;
        return Map::from_reader(&mut reader);
    }
}
