use std::fs;
use std::path;

use std::error::{ Error };
use std::io::{Read, Seek };

use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub rows: usize,
    pub columns: usize,
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
                Shape { rows: 2, columns: 3 }
            },
            14..=18|24 => {
                Shape { rows: 2, columns: 2 }
            },
            19 => {
                Shape { rows: 3, columns: 3 }
            },
            20|21 => {
                Shape { rows: 2, columns: 3 }
            },
            _ => {
                Shape { rows: 1, columns: 1 }
            },
        }
    }
}

pub struct Tilemap {
    pub shape: Shape,
    pub tiles: Vec<usize>,
}

impl Tilemap {
    pub fn surface(
        &self,
        palette: &Palette,
        tileset: &Tileset,
    ) -> Surface {
        let tiles = self.tiles.iter()
            .map(|&tile_index| tileset.surface(tile_index, palette))
            .collect::<Vec<_>>();

        let width = tileset.tile_size.width*self.shape.columns as u32;
        let height = tileset.tile_size.height*self.shape.rows as u32;

        let mut surface = Surface::new(Size {
            width,
            height,
        });

        for (i, tile) in tiles.iter().enumerate() {
            let row = (i/self.shape.columns) as u32;
            let column = (i%self.shape.columns) as u32;

            let dst_rect = Rect::from_point_and_size(Point {
                x: (column*tileset.tile_size.width) as i32,
                y: (row*tileset.tile_size.height) as i32,
            }, tile.size());

            surface.blit(tile, tile.rect(), dst_rect);
        }

        surface
    }
}

impl Tilemap {
    pub fn from_map_reader<T>(
        reader: &mut T,
    ) -> Result<Vec<Tilemap>, Box<dyn Error>> where T: Read + Seek {
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
        let tilemaps =
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
                    icons.push(Tilemap {
                        shape,
                        tiles,
                    })
                }

                icons
            })
            .collect::<Vec<_>>();

        Ok(tilemaps)
    }

    pub fn from_map_file<P>(
        path: P,
    ) -> Result<Vec<Tilemap>, Box<dyn Error>> where P: AsRef<path::Path> {
        let mut reader = fs::File::open(path)?;
        Self::from_map_reader(&mut reader)
    }
}
