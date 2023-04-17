use std::error::Error;
use std::fs;
use std::path::PathBuf;

use crate::config::Cli;

fn export_tile_to_bmp(
    tile: &dune2::ICNTile,
    palette: &dune2::Palette,
    output_filepath: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let surface = tile.surface(&palette);
    let mut output = fs::File::create(output_filepath)?;
    dune2::write_bmp_with_palette(&surface, &palette, &mut output)?;
    return Ok(());
}

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    let output_dirpath = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(&config.shp_input_filepath.file_stem().unwrap())
    });

    fs::create_dir_all(&output_dirpath)?;

    let palette = dune2::Palette::try_from(config.pal_input_filepath)?;
    let icn = dune2::ICN::try_from(config.shp_input_filepath)?;

    if let Some(map_input_filepath) = config.map_input_filepath {
        let map = dune2::Map::try_from(map_input_filepath)?;

        return Ok(());
    } else {
        for (i, frame) in icn.tiles.iter().enumerate() {
            let output_filepath = output_dirpath.join(format!("{:02}.bmp", i));
            export_tile_to_bmp(
                frame,
                &palette,
                &output_filepath
            )?;
        }
    }

    return Ok(());
}
