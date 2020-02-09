fn main() {
    let text = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let doc = roxmltree::Document::parse(&text).unwrap();

    let mut elements = 0;
    let mut attributes = 0;
    for node in doc.root().descendants().filter(|n| n.is_element()) {
        elements += 1;
        attributes += node.attributes().len();
    }

    println!("Elements: {}", elements);
    println!("Attributes: {}", attributes);
}
