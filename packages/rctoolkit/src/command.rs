use std::error::Error;
use std::path::PathBuf;

use std::fs;

use dune2::Dune2RC;
use rmp_serde::{Serializer};
use serde::{Serialize};
use toml;

use crate::config::{Cli, Config};

pub fn load_rc_config(
    config_filepath: &PathBuf,
) -> Result<Config, Box<dyn Error>> {
    let config_str = fs::read_to_string(config_filepath)?;
    let config = toml::from_str::<Config>(&config_str).map(|mut config| {
        let mut palette = config_filepath.clone();

        palette.pop();
        palette.push(config.palette);

        config.palette = palette;

        config
    })?;

    Ok(config)
}

pub fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let config = load_rc_config(&cli.config_filepath)?;

    let palette = dune2::Palette::from_pal_file(&config.palette)?;
    let rc = Dune2RC {
        palette,
    };

    if cli.output_file.exists() && !cli.overwrite {
        return Err("Output file already exists. Use --force to overwrite.".into());
    }

    let mut output = fs::File::create(&cli.output_file)?;
    let mut serializer = Serializer::new(&mut output);

    rc.serialize(&mut serializer)?;

    Ok(())
}
