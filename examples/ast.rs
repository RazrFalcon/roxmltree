fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage:\n\tcargo run --example ast -- input.xml");
        std::process::exit(1);
    }

    let text = std::fs::read_to_string(&args[1]).unwrap();

    // Allow DTD for this example.
    let opt = roxmltree::ParsingOptions {
        allow_dtd: true,
    };

    match roxmltree::Document::parse_with_options(&text, opt) {
        Ok(doc) => print!("{:?}", doc),
        Err(e) => println!("Error: {}.", e),
    }
}
