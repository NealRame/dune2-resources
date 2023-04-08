use std::error::Error;
use std::fs;

use std::path::PathBuf;

use crate::config::Cli;


pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    let output_dirpath = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(&config.shp_input_filepath.file_stem().unwrap())
    });

    fs::create_dir_all(&output_dirpath)?;

    let palette = dune2::Palette::try_from(config.pal_input_filepath)?;
    let shp = dune2::SHP::try_from(config.shp_input_filepath)?;

    for (i, frame) in shp.frames.iter().enumerate() {
        let surface = frame.surface(&palette);
        let output_filepath = output_dirpath.join(format!("{}.bmp", i));
        let mut output = fs::File::create(output_filepath)?;
        dune2::write_bmp_with_palette(&surface, &palette, &mut output)?;
    }

    return Ok(());
}
