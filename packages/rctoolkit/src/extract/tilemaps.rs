use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;

use dune2_rc::prelude::{
    bitmap_blit,
    Bitmap,
    Resources,
    TilemapBitmap,
};

use crate::image::BMPImage;


#[derive(clap::Args)]
pub struct Args {
    /// Output folder path
    #[arg(short = 'd', long)]
    pub output_dir: Option<PathBuf>,

    /// Faction to export.
    #[arg(short = 'F', long)]
    pub faction: Option<super::cli_config::ArgExtractFaction>,

    /// Scale factor
    #[arg(short = 's', long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..))]
    pub scale: u32,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

pub fn extract(
    rc: &Resources,
    args: &Args,
) -> Result<()> {
    let faction = args.faction.map(|f| f.into());
    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tilemaps")?);

    fs::create_dir_all(&output_dir)?;

    for (i, tilemap) in rc.tilemaps.iter().enumerate() {
        let output_filepath = output_dir.join(format!("{:03}.bmp", i));

        let bitmap = TilemapBitmap::try_with_resources(tilemap, faction, rc)?;
        let src_rect = bitmap.rect();

        let mut image = BMPImage::new(args.scale*bitmap.size());
        let dst_rect = image.rect();

        bitmap_blit(&bitmap, &src_rect, &mut image, &dst_rect);
        image.save(output_filepath)?;
    }

    return Ok(());
}
