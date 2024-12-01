use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;

use dune2_assets::prelude::{
    bitmap_blit,
    Color,
    Bitmap,
    Assets,
    TilemapBitmap,
};

use crate::image::BMPImageBuilder;


#[derive(clap::Args)]
pub struct Args {
    /// Background color. BACKGROUND_COLOR can be any valid css color string
    #[arg(short = 'b', long, value_parser = clap::value_parser!(Color), default_value = "black")]
    pub background_color: Color,

    /// Faction to export.
    #[arg(short = 'F', long)]
    pub faction: Option<super::cli_config::ArgExtractDune2Faction>,

    /// Scale factor
    #[arg(short = 's', long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..))]
    pub scale: u32,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,

    /// Output folder path
    #[arg(short = 'd', long)]
    pub output_dir: Option<PathBuf>,
}

pub fn extract(
    rc: &Assets,
    args: &Args,
) -> Result<()> {
    let faction = args.faction.map(|f| f.into());
    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tilemaps")?);

    fs::create_dir_all(&output_dir)?;

    for (i, tilemap) in rc.tilemaps.iter().enumerate() {
        let output_filepath = output_dir.join(format!("{:03}.bmp", i));

        let bitmap = TilemapBitmap::try_with_assets(tilemap, faction, rc)?;
        let src_rect = bitmap.rect();

        let mut image = BMPImageBuilder::new(
            args.scale*bitmap.size()
        ).with_background_color(args.background_color).build();
        let dst_rect = image.rect();

        bitmap_blit(&bitmap, &src_rect, &mut image, &dst_rect);
        image.save(output_filepath)?;
    }

    return Ok(());
}
