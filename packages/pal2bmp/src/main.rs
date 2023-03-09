mod config;
mod command;

use std::env;

fn main() {
    let config = config::build(env::args())
        .unwrap_or_else(|err| {
            println!("Error: {}", err);
            std::process::exit(1);
        });

    command::run(config).unwrap_or_else(|err| {
        println!("Error: {}", err);
        std::process::exit(1);
    });
}
