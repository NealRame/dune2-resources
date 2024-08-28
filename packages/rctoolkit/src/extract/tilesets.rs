use std::fs;
use std::path::{
    Path,
    PathBuf,
};
use std::str::FromStr;

use anyhow::Result;

use dune2_rc::{
    bitmap,
    constants,
    Bitmap,
    Point,
    Resources,
    Rect,
    Size,
    TileBitmap,
};

use crate::image::BMPImage;


#[derive(clap::Args)]
pub struct Args {
    /// Tileset id to extract. If not specified all tileset will be extracted.
    pub tileset_id: Option<String>,

    /// Faction to export.
    #[arg(short = 'F', long)]
    pub faction: Option<super::cli_config::ArgExtractFaction>,

    /// Overwrite existing files.
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,

    /// Multiple
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub multiple: bool,

    /// Output folder path.
    #[arg(short = 'd', long)]
    pub output_dir: Option<PathBuf>,

    /// Scale factor.
    #[arg(short = 's', long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..))]
    pub scale: u32,
}

fn extract_tileset_image(
    rc: &Resources,
    tileset_id: &str,
    faction: Option<constants::Faction>,
    scale: u32,
    base_output_dir: &Path,
) -> Result<()> {
    let output_file = base_output_dir.join(format!("{tileset_id}.bmp"));

    let tileset = rc.get_tileset(tileset_id)?;

    let tile_count = tileset.tile_count();
    let tile_size = tileset.tile_size();

    let cols = 16;
    let rows = if tile_count%16 == 0 {
        tile_count/16
    } else {
        tile_count/16 + 1
    } as u32;

    let image_size = Size {
        width: cols*scale*tile_size.width,
        height: rows*scale*tile_size.height,
    };

    let mut image = BMPImage::new(image_size);

    for (index, tile) in tileset.tile_iter().enumerate() {
        let col = (index%16) as i32;
        let row = (index/16) as i32;

        let bitmap = TileBitmap::with_resources(tile, faction, rc);
        let src_rect = bitmap.rect();

        let dst_rect = Rect::from_point_and_size(
            Point {
                x: col*(scale*tile_size.width) as i32,
                y: row*(scale*tile_size.height) as i32,
            },
            Size {
                width: scale*tile_size.width,
                height: scale*tile_size.height,
            },
        );

        bitmap::blit(&bitmap, &src_rect, &mut image, &dst_rect);
    }

    image.save(output_file)?;

    Ok(())
}

fn extract_tileset_tiles(
    rc: &Resources,
    tileset_id: &str,
    faction: Option<constants::Faction>,
    scale: u32,
    base_output_dir: &Path,
) -> Result<()> {
    let output_dir = base_output_dir.join(tileset_id);

    let tileset = rc.get_tileset(tileset_id)?;

    let tile_count = tileset.tile_count();
    let tile_index_width = if tile_count > 0 {
        f32::log10(tile_count as f32) as usize + 1
    } else {
        1
    };

    fs::create_dir_all(&output_dir)?;

    for (tile_index, tile) in tileset.tile_iter().enumerate() {
        let filename = format!("{:01$}.bmp", tile_index, tile_index_width);

        let bitmap = TileBitmap::with_resources(tile, faction, rc);
        let src_rect = bitmap.rect();

        let mut image = BMPImage::new(scale*bitmap.size());
        let dst_rect = image.rect();

        bitmap::blit(&bitmap, &src_rect, &mut image, &dst_rect);
        image.save(output_dir.join(filename))?;
    }

    Ok(())
}

fn extract_tileset(
    rc: &Resources,
    args: &Args,
    tileset_id: &str,
) ->  Result<()> {
    let faction = args.faction.map(|f| f.into());
    let base_output_dir = if let Some(dir) = args.output_dir.as_ref() {
        PathBuf::clone(dir)
    } else {
        PathBuf::from_str("tilesets")?
    };
    let scale = args.scale;

    fs::create_dir_all(&base_output_dir)?;

    if args.multiple {
        extract_tileset_tiles(rc, tileset_id, faction, scale, &base_output_dir)
    } else {
        extract_tileset_image(rc, tileset_id, faction, scale, &base_output_dir)
    }
}

pub fn extract(
    rc: &Resources,
    args: &Args,
) -> Result<()> {
    if let Some(tileset_id) = &args.tileset_id {
        extract_tileset(rc, args, tileset_id)?;
    } else {
        for tileset in rc.tilesets.values() {
            extract_tileset(rc, args, tileset.get_id())?;
        }
    }

    return Ok(());
}
