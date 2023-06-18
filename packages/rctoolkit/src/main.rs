mod image;
mod create;
mod extract;

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Create(create::Cli),
    Extract(extract::Cli),
}

#[derive(Parser)]
#[command(author, about, version)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

fn main() {
    let cli = Cli::parse();

    let res = match &cli.command {
        Commands::Create(args) => create::run(args),
        Commands::Extract(args) => extract::run(args),
    };

    if let Err(err) = res {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}
