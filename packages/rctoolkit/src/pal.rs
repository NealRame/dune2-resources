use std::fs;
use std::io;
use std::path;

use anyhow::{anyhow, Result};

use dune2_rc::prelude::{
    Color,
    Palette,
};


fn read_palette_from_reader(
    reader: &mut impl io::Read,
) -> Result<Palette> {
    let mut palette = Palette::new();
    let mut buf = [0; 3];

    loop {
        let color = match reader.read(&mut buf)? {
            0 => break,
            3 => Color::from(&buf),
            _ => return Err(anyhow!("Invalid palette file")),
        };
        // We have to multiply each channel by 4 because the palette is 6
        // bits per channel
        palette.push(&(4*color));
    }

    Ok(palette)
}

pub fn read_palette_from_file(
    path: &path::PathBuf,
) -> Result<Palette> {
    let mut reader = fs::File::open(path)?;
    read_palette_from_reader(&mut reader)
}
