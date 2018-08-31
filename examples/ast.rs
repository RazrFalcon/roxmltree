extern crate roxmltree;

use std::fs;
use std::env;
use std::io::Read;
use std::process;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("Usage:\n\tcargo run --example ast -- input.xml");
        process::exit(1);
    }

    let text = load_file(&args[1]);
    match roxmltree::Document::parse(&text) {
        Ok(doc) => print!("{:?}", doc),
        Err(e) => println!("Error: {}.", e),
    }
}

fn load_file(path: &str) -> String {
    let mut file = fs::File::open(&path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}
