mod config;
mod unpak;

use std::env;
use crate::config::*;
use crate::unpak::*;

fn main() {
    let config = Config::build(env::args())
        .unwrap_or_else(|err| {
            println!("Error: {}", err);
            std::process::exit(1);
        });

    unpak(config).unwrap_or_else(|err| {
        println!("Error: {}", err);
        std::process::exit(1);
    });
}
