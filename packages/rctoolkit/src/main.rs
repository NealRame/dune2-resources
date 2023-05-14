mod cli;
mod create;

use clap::Parser;

use cli::*;

fn main() {
    let cli = Cli::parse();

    let res = match &cli.command {
        Commands::Create(args) => create::run(args),
    };

    if let Err(err) = res {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}
