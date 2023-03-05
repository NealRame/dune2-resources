use std::env;
use std::path::Path;
use std::path::PathBuf;

pub struct Config {
    pub input_filepath: PathBuf,
    pub output_dirpath: PathBuf,
}

fn get_input_filepath(
    args: &mut impl Iterator<Item = String>
) -> Result<PathBuf, &'static str> {
    match args.next() {
        Some(arg) => Ok(PathBuf::from(arg)),
        None => Err("No input pak file specified"),
    }
}

fn get_output_dirpath(
    args: &mut impl Iterator<Item = String>,
    input_filepath: &Path,
) -> Result<PathBuf, &'static str> {
    if let Some(arg) = args.next() {
        return Ok(PathBuf::from(arg));
    }

    if let Some(arg) = env::var("UNPAK_OUTPUT_DIR").ok() {
        return Ok(PathBuf::from(arg));
    }

    if let Some(file_stem) = Path::new(&input_filepath).file_stem() {
        return Ok(PathBuf::from(file_stem));
    }

    Err("No output directory specified")
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>)
        -> Result<Config, &'static str>
    {
        args.next(); // skip the first arguments as it is the program name

        let input_filepath = get_input_filepath(&mut args)?;
        let output_dirpath = get_output_dirpath(&mut args, &input_filepath)?;

        Ok(Config {
            input_filepath,
            output_dirpath,
        })
    }
}