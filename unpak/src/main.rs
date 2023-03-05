mod config;
mod cmd;

use std::env;

fn main() {
    let config = config::build(env::args())
        .unwrap_or_else(|err| {
            println!("Error: {}", err);
            std::process::exit(1);
        });

    cmd::run(config).unwrap_or_else(|err| {
        println!("Error: {}", err);
        std::process::exit(1);
    });
}
