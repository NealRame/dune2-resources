use std::error::Error;
use std::fs;
use std::path::PathBuf;

use crate::config::Cli;

fn export_tilemap_to_bmp(
    tilemap: &dune2::Tilemap,
    tileset: &dune2::Tileset,
    palette: &dune2::Palette,
    output_filepath: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let surface = tilemap.surface(&palette, &tileset);
    let mut output = fs::File::create(output_filepath)?;
    dune2::write_bmp_with_palette(&surface, &palette, &mut output)?;
    return Ok(());
}

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    let output_dirpath = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(&config.icn_input_filepath.file_stem().unwrap())
    });

    fs::create_dir_all(&output_dirpath)?;

    let palette = dune2::Palette::from_pal_file(&config.pal_input_filepath)?;
    let tileset = dune2::Tileset::from_icn_file(&config.icn_input_filepath)?;

    let tilemaps = if let Some(map_input_filepath) = config.map_input_filepath {
        dune2::Tilemap::from_map_file(&map_input_filepath)?
    } else {
        Vec::from_iter((0..tileset.tiles.len()).map(|i| {
            dune2::Tilemap {
                shape: dune2::Shape { rows: 1, columns: 1, },
                tiles: vec![i],
            }
        }))
    };

    for (i, tilemap) in tilemaps.iter().enumerate() {
        let output_filepath = output_dirpath.join(format!("{:02}.bmp", i));
        export_tilemap_to_bmp(
            &tilemap,
            &tileset,
            &palette,
            &output_filepath
        )?;
    }

    return Ok(());
}
