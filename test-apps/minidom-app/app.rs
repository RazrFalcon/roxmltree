fn main() {
    let text = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let root: minidom::Element = text.parse().unwrap();

    let mut elements = 1;
    let mut attributes = root.attrs().count();
    count(&root, &mut elements, &mut attributes);

    println!("Elements: {}", elements);
    println!("Attributes: {}", attributes);
}

fn count(parent: &minidom::Element, elements: &mut usize, attributes: &mut usize) {
    for elem in parent.children() {
        *elements += 1;
        *attributes += elem.attrs().count();

        if elem.children().count() != 0 {
            count(&elem, elements, attributes);
        }
    }
}
