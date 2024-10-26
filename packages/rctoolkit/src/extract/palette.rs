use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

use dune2_rc::prelude::{
    bitmap_fill_rect,
    Point,
    Resources,
    Rect,
    Size,
};

use crate::image::BMPImageBuilder;


#[derive(clap::Args)]
pub struct Args {
    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,

    /// Output folder path
    #[arg(short, long, default_value = "palette.bmp")]
    pub output_filepath: PathBuf,
}

pub fn extract(
    rc: &Resources,
    args: &Args,
) -> Result<()> {
    if let Some(parent) = args.output_filepath.parent() {
        fs::create_dir_all(parent)?;
    }

    let palette_watch_size = Size { width: 32, height: 32 };
    let mut palette_image = BMPImageBuilder::new(Size {
        width: 32*16,
        height: 32*((rc.palette.len() as f32)/16.).ceil() as u32,
    }).build();

    for (i, color) in rc.palette.iter() {
        let rect = Rect::from_point_and_size(
            32*Point {
                x: (i as i32)%16,
                y: (i as i32)/16,
            },
            palette_watch_size,
        );

        bitmap_fill_rect(&mut palette_image, &rect, Some(color));
    }

    if args.output_filepath.exists() && !args.force_overwrite {
        return Err(anyhow!(
            "Output file already exists. Use --force to overwrite."
        ));
    }

    palette_image.save(&args.output_filepath)?;

    Ok(())
}
