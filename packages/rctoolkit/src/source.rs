use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

use dune2_rc::{bitmap, Bitmap, TileBitmap};

use crate::image::BMPImage;
use crate::resources_config::*;


/******************************************************************************
 * Source run
 *****************************************************************************/

#[derive(clap::Args)]
pub struct Args {
    /// Input file path
    pub config_filepath: PathBuf,

    /// Output folder path
    #[arg(long, short, default_value = "sources")]
    pub output_dir: PathBuf,

    /// Scale factor
    #[arg(short = 's', long, value_parser = clap::value_parser!(u32).range(1..))]
    pub scale: Option<u32>,
}

pub fn run(args: &Args) -> Result<()> {
    let config = Config::try_read_from_file(&args.config_filepath)?;
    let scale = args.scale.unwrap_or(1);

    let palette = config.load_palette()?;
    let sources = config.load_sources()?;

    if args.output_dir.exists() {
        return Err(anyhow!("Output file already exists."));
    }

    fs::create_dir_all(&args.output_dir)?;

    let tile_count = sources.len();
    let tile_index_width = if tile_count > 0 {
        f32::log10(tile_count as f32) as usize + 1
    } else {
        1
    };

    for (tile_index, tile) in sources.iter().enumerate() {
        let tile_size = tile.size();
        let filename = format!("{:01$}_{tile_size}.bmp", tile_index, tile_index_width);

        let bitmap = TileBitmap::with_palette(tile, None, &palette);

        let src_rect = bitmap.rect();

        let mut image = BMPImage::new(scale*bitmap.size());
        let dst_rect = image.rect();

        bitmap::blit(&bitmap, &src_rect, &mut image, &dst_rect);
        image.save(args.output_dir.join(filename))?;
    }

    Ok(())
}