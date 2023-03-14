use std::error::Error;
use std::fs;

use crate::config::Config;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = config.output_filepath.parent() {
        fs::create_dir_all(parent)?;
    }

    let palette = bitmap::Palette::load(&mut fs::File::open(&config.input_filepath)?)?;
    let palette_width = 32*16;
    let palette_height = 32*((palette.len() as f32)/16.).ceil() as u32;

    let mut palette_bitmap = bitmap::Bitmap::new(palette_width, palette_height);

    let palette_color_watch_size = bitmap::Size { width: 32, height: 32 };

    for (i, color) in palette.iter() {
        let rect = bitmap::Rect::from_point_and_size(
            32*bitmap::Point {
                x: (i as i32)%16,
                y: (i as i32)/16,
            },
            palette_color_watch_size,
        );

        palette_bitmap.fill_rect(&rect, color);
    }

    let mut output = fs::File::create(&config.output_filepath)?;
    palette_bitmap.write_with_palette(&mut output, &palette)?;

    Ok(())
}