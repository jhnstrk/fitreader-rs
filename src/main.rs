
// std imports
use std::env;

#[macro_use] extern crate log;

extern crate fit_reader;
use crate::fit_reader::fitfile;

extern crate env_logger;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        warn!("Usage: fit_file input_file.fit");
        return;
    }

    env_logger::init();

    let res = fitfile::read_file(args.get(1).unwrap());

    match res {
        Ok(_) => {},
        Err(e) => warn!("Error: {:?}", e),
    };
    info!("Done");
}
