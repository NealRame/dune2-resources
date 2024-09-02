mod check;
mod create;
mod extract;
mod icn;
mod image;
mod info;
mod io;
mod resources_config;
mod shp;
mod source;


use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Check(check::Args),
    Create(create::Args),
    Source(source::Args),
    Extract(extract::Args),
    Info(info::Args),
}

#[derive(Parser)]
#[command(author, about, version)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

fn main() {
    let args = Args::parse();

    let res = match &args.command {
        Commands::Check(args) => check::run(args),
        Commands::Create(args) => create::run(args),
        Commands::Source(args) => source::run(args),
        Commands::Extract(args) => extract::run(args),
        Commands::Info(args) => info::run(args),
    };

    if let Err(err) = res {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}
