fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage:\n\tcargo run --example ast -- input.xml");
        std::process::exit(1);
    }

    let text = std::fs::read_to_string(&args[1]).unwrap();
    match roxmltree::Document::parse(&text) {
        Ok(doc) => print!("{:?}", doc),
        Err(e) => println!("Error: {}.", e),
    }
}
