fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage:\n\tcargo run --example print_pos -- input.xml");
        std::process::exit(1);
    }

    let text = std::fs::read_to_string(&args[1]).unwrap();
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
            println!("{:?} at {}", node.tag_name(), doc.text_pos_at(node.range().start));
        }
    }
}
