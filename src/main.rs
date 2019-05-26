
// std imports
use std::env;

extern crate fit_reader;
use crate::fit_reader::fitfile;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("Usage: fit_file input_file.fit");
        return;
    }
    let res = fitfile::read_file(args.get(1).unwrap());

    match res {
        Ok(_) => {},
        Err(e) => println!("Error: {:?}", e),
    };
    println!("Done");
}
