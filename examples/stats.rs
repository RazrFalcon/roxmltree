extern crate roxmltree;

use std::collections::HashSet;
use std::fs;
use std::env;
use std::io::Read;
use std::process;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("Usage:\n\tcargo run --example stats -- input.xml");
        process::exit(1);
    }

    let text = load_file(&args[1]);
    let doc = match roxmltree::Document::parse(&text) {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}.", e);
            process::exit(1);
        }
    };

    println!("Elements count: {}",
             doc.root().descendants().filter(|n| n.is_element()).count());

    let attrs_count: usize = doc.root().descendants().map(|n| n.attributes().len()).sum();
    println!("Attributes count: {}", attrs_count);

    let ns_count: usize = doc.root().descendants().map(|n| n.namespaces().len()).sum();
    println!("Namespaces count: {}", ns_count);

    let mut uris = HashSet::new();
    for node in doc.root().descendants() {
        for ns in node.namespaces() {
            uris.insert((ns.name().to_string(), ns.uri().to_string()));
        }
    }
    println!("Unique namespaces count: {}", uris.len());
    if !uris.is_empty() {
        println!("Unique namespaces:");
        for (key, value) in uris {
            println!("  {:?}: {}", key, value);
        }
    }

    println!("Comments count: {}",
             doc.root().descendants().filter(|n| n.is_comment()).count());

    println!("Comments:");
    for node in doc.root().descendants().filter(|n| n.is_comment()) {
        println!("{:?}", node.text().unwrap());
    }
}

fn load_file(path: &str) -> String {
    let mut file = fs::File::open(&path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}
