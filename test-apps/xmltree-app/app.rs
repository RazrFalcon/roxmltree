fn main() {
    let data = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let root = xmltree::Element::parse(data.as_slice()).unwrap();

    let mut elements = 1;
    let mut attributes = root.attributes.len();
    count(&root, &mut elements, &mut attributes);

    println!("Elements: {}", elements);
    println!("Attributes: {}", attributes);
}

fn count(parent: &xmltree::Element, elements: &mut usize, attributes: &mut usize) {
    for child in &parent.children {
        if let xmltree::XMLNode::Element(elem) = child {
            *elements += 1;
            *attributes += elem.attributes.len();

            if !elem.children.is_empty() {
                count(&elem, elements, attributes);
            }
        }
    }
}
