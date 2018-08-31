extern crate roxmltree;

use std::fs;
use std::env;
use std::io::Read;
use std::process;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("Usage:\n\tcargo run --example print_pos -- input.xml");
        process::exit(1);
    }

    let text = load_file(&args[1]);
    let doc = match roxmltree::Document::parse(&text) {
        Ok(doc) => doc,
        Err(e) => {
            println!("Error: {}.", e);
            return;
        },
    };

    // TODO: finish
    for node in doc.descendants() {
        if node.is_element() {
            println!("{:?} at {}", node.tag_name(), node.node_pos());
        }
    }
}

fn load_file(path: &str) -> String {
    let mut file = fs::File::open(&path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}
