fn main() {
    let text = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let package = sxd_document::parser::parse(&text).unwrap();
    let doc = package.as_document();

    let mut elements = 0;
    let mut attributes = 0;
    for child in doc.root().children() {
        if let sxd_document::dom::ChildOfRoot::Element(elem) = child {
            elements += 1;
            attributes += elem.attributes().len();
            count(&elem, &mut elements, &mut attributes);
        }
    }

    println!("Elements: {}", elements);
    println!("Attributes: {}", attributes);
}

fn count(parent: &sxd_document::dom::Element, elements: &mut usize, attributes: &mut usize) {
    for child in parent.children() {
        if let sxd_document::dom::ChildOfElement::Element(elem) = child {
            *elements += 1;
            *attributes += elem.attributes().len();

            if !elem.children().is_empty() {
                count(&elem, elements, attributes);
            }
        }
    }
}
