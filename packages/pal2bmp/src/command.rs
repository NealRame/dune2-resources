use std::error::Error;
use std::fs;

use crate::config::Cli;

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = config.output_filepath.parent() {
        fs::create_dir_all(parent)?;
    }

    let palette = dune2::Palette::load(&mut fs::File::open(&config.input_filepath)?)?;
    let palette_watch_size = dune2::Size { width: 32, height: 32 };
    let palette_size = dune2::Size {
        width: 32*16,
        height: 32*((palette.len() as f32)/16.).ceil() as u32,
    };

    let mut palette_surface = dune2::Surface::new(palette_size);

    for (i, color) in palette.iter() {
        let rect = dune2::Rect::from_point_and_size(
            32*dune2::Point {
                x: (i as i32)%16,
                y: (i as i32)/16,
            },
            palette_watch_size,
        );
        palette_surface.fill_rect(&rect, color);
    }

    if config.output_filepath.exists() && !config.overwrite {
        return Err("Output file already exists. Use --force to overwrite.".into());
    }

    let mut output = fs::File::create(&config.output_filepath)?;
    dune2::write_bmp_with_palette(&palette_surface, &palette, &mut output)?;

    Ok(())
}