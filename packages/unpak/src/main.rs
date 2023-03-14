mod command;
mod config;

use clap::Parser;
use config::Cli;

fn main() {
    let config = Cli::parse();

    command::run(config).unwrap_or_else(|err| {
        println!("Error: {}", err);
        std::process::exit(1);
    });
}
