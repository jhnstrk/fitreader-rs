
// std imports
use std::env;

#[macro_use] extern crate log;

extern crate fit_reader;
use crate::fit_reader::fitfile;

extern crate env_logger;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("Usage: fit_file input_file.fit");
        std::process::exit(1);
    }

    for pathname in &args[1..] {
        info!("Processing {}", pathname);
        let res = fitfile::read_file(&pathname);

        match res {
            Ok(_) => {},
            Err(e) => {
                error!("Error: {:?}", e);
                std::process::exit(1);
            },
        };
    }

    info!("Done");
    std::process::exit(0);
}
