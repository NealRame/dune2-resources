mod command;
mod config;

use std::io::ErrorKind;

use clap::Parser;
use config::Cli;

fn main() {
    let config = Cli::parse();

    command::run(config).unwrap_or_else(|err| {
        if err.is::<std::io::Error>() {
            let io_err = err.downcast::<std::io::Error>().unwrap();

            match io_err.kind() {
                ErrorKind::AlreadyExists => {
                    eprintln!("File '{}' already exists!", io_err.to_string());
                    eprintln!("Use '-f' option to force file overwrite.");
                },
                _ => {
                    eprintln!("IOError: {}", io_err);
                }
            }
        } else {
            println!("Error: {}", err);
        }
        std::process::exit(1);
    });
}
