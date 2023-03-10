use std::error::Error;

use std::fs;
use std::io;

use crate::config::Config;

struct PaletteColorReader<T>
    where T: io::Read {
    source: T,
}

impl<T> PaletteColorReader<T>
    where T: io::Read {
    fn new(source: T) -> PaletteColorReader<T> {
        PaletteColorReader { source }
    }
}

impl<T> Iterator for PaletteColorReader<T>
    where T: io::Read {
    type Item = bitmap::Color;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0; 3];
        match self.source.read(&mut buf) {
            Ok(3) => Some(bitmap::Color::new(4*buf[0], 4*buf[1], 4*buf[2])),
            _ => None,
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = config.output_filepath.parent() {
        fs::create_dir_all(parent)?;
    }

    let palette = PaletteColorReader::new(fs::File::open(&config.input_filepath)?).collect::<Vec<bitmap::Color>>();

    let palette_width = 32*16;
    let palette_height = 32*((palette.len() as f32)/16.).ceil() as u32;
    let mut palette_bitmap = bitmap::Bitmap::new(palette_width, palette_height);

    let color_watch_size = bitmap::Size { width: 32, height: 32 };
 
    for (i, color) in (0_i32..).zip(palette.iter()) {
        let color_watch_pos = 32*bitmap::Point {
            x: i%16,
            y: i/16,
        };

        let rect = bitmap::Rect::from_point_and_size(
            color_watch_pos,
            color_watch_size,
        );

        palette_bitmap.fill_rect(&rect, *color);
    }

    let mut output = fs::File::create(&config.output_filepath)?;

    palette_bitmap.write(&mut output)?;

    Ok(())
}