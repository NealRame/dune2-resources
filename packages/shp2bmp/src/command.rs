use std::error::Error;
use std::fs;

use std::path::PathBuf;
use std::str::FromStr;

use dune2::Faction;

use crate::config::Cli;

fn export_frame_to_bmp(
    frame: &dune2::SHPFrame,
    palette: &dune2::Palette,
    faction: Faction,
    output_filepath: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let surface = frame.surface(&palette, faction);
    let mut output = fs::File::create(output_filepath)?;
    dune2::write_bmp_with_palette(&surface, &palette, &mut output)?;
    return Ok(());
}

fn export_frame_remap_tanble_to_bmp(
    remap_table: &Vec<usize>,
    palette: &dune2::Palette,
    output_filepath: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let remap_table_len = remap_table.len() as u32;

    let palette_watch_size = dune2::Size { width: 32, height: 32 };
    let mut palette_surface = dune2::Surface::new(dune2::Size {
        width: 32*16.min(remap_table_len) as u32,
        height: 32*((remap_table_len as f32)/16.).ceil() as u32,
    });

    for (i, color_index) in remap_table.iter().enumerate() {
        let rect = dune2::Rect::from_point_and_size(
            32*dune2::Point {
                x: (i as i32)%16,
                y: (i as i32)/16,
            },
            palette_watch_size,
        );
        palette_surface.fill_rect(&rect, palette.color_at(*color_index));
    }

    let mut output = fs::File::create(&output_filepath)?;
    dune2::write_bmp_with_palette(&palette_surface, &palette, &mut output)?;

    return Ok(());
}

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    let output_dirpath = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(&config.shp_input_filepath.file_stem().unwrap())
    });

    fs::create_dir_all(&output_dirpath)?;

    let faction = Faction::from_str(&config.faction)?;
    let palette = dune2::Palette::try_from(config.pal_input_filepath)?;
    let shp = dune2::SHP::try_from(config.shp_input_filepath)?;

    for (i, frame) in shp.frames.iter().enumerate() {
        let output_filepath = output_dirpath.join(format!("{:02}.bmp", i));
        export_frame_to_bmp(
            frame,
            &palette,
            faction,
            &output_filepath
        )?;

        if config.export_remap_table {
            let output_filepath = output_dirpath.join(format!("{:02}_remap_table.bmp", i));
            export_frame_remap_tanble_to_bmp(
                &frame.remap_table,
                &palette, &output_filepath
            )?;
        }
    }

    return Ok(());
}
