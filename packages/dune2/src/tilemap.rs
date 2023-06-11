use std::fs;
use std::path;

use std::collections::HashMap;
use std::error::{Error};
use std::io::{Read, Seek};

use serde::{Deserialize, Serialize};

use crate::shape::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tilemap {
    pub shape: Shape,
    pub tiles: Vec<usize>,
}

impl Tilemap {
    pub fn from_map_reader<T>(
        reader: &mut T,
        shapes: &HashMap<String, Shape>,
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
                let start = *start;
                let end = *end;
                (start, if end == 0 { indexes.len() } else { end })
            })
            .enumerate()
            .flat_map(|(icon_index, (start, end))| {
                let count = end - start;

                let shape_key = format!("{}", icon_index);
                let shape_fallback = Shape { rows: 1, columns: count as u32 };
                let shape = shapes
                    .get(&shape_key)
                    .unwrap_or(&shape_fallback);

                let shape_size = (shape.rows*shape.columns) as usize;
                let shape_count = count/shape_size;

                let mut icons = Vec::with_capacity(shape_count);
                for i in 0..shape_count {
                    let start = start + i*shape_size;
                    let end = start + shape_size;
                    let tiles = indexes[start..end].to_vec();
                    icons.push(Tilemap {
                        shape: *shape,
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
        shapes: &HashMap<String, Shape>,
    ) -> Result<Vec<Tilemap>, Box<dyn Error>> where P: AsRef<path::Path> {
        let mut reader = fs::File::open(path)?;
        Self::from_map_reader(&mut reader, shapes)
    }
}
