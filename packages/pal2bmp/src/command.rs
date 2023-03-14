use std::error::Error;
use std::fs;

use crate::config::Cli;

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = config.output_filepath.parent() {
        fs::create_dir_all(parent)?;
    }

    let palette = dune2::Palette::load(&mut fs::File::open(&config.input_filepath)?)?;
    let palette_width = 32*16;
    let palette_height = 32*((palette.len() as f32)/16.).ceil() as u32;

    let mut palette_bitmap = dune2::Bitmap::new(palette_width, palette_height);

    let palette_color_watch_size = dune2::Size { width: 32, height: 32 };

    for (i, color) in palette.iter() {
        let rect = dune2::Rect::from_point_and_size(
            32*dune2::Point {
                x: (i as i32)%16,
                y: (i as i32)/16,
            },
            palette_color_watch_size,
        );

        palette_bitmap.fill_rect(&rect, color);
    }

    if config.output_filepath.exists() && !config.overwrite {
        return Err("Output file already exists. Use --force to overwrite.".into());
    }

    let mut output = fs::File::create(&config.output_filepath)?;
    palette_bitmap.write_with_palette(&mut output, &palette)?;

    Ok(())
}