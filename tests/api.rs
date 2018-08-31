extern crate roxmltree;
#[macro_use] extern crate pretty_assertions;

use roxmltree::*;

#[test]
fn root_element_01() {
    let data = "\
<!-- comment -->
<e/>
";

    let doc = Document::parse(data).unwrap();
    let node = doc.root_element();
    assert_eq!(node.tag_name().name(), "e");
}

#[test]
fn get_text_01() {
    let data = "\
<root>
    Text1
    <item>
        Text2
    </item>
    Text3
</root>
";

    let doc = Document::parse(data).unwrap();
    let root = doc.root_element();

    assert_eq!(root.text(), Some("\n    Text1\n    "));
    assert_eq!(root.tail(), None);

    let item = root.children().nth(1).unwrap();

    assert_eq!(item.text(), Some("\n        Text2\n    "));
    assert_eq!(item.tail(), Some("\n    Text3\n"));
}

#[test]
fn get_text_02() {
    let data = "<root>&apos;</root>";

    let doc = Document::parse(data).unwrap();
    let root = doc.root_element();

    assert_eq!(root.text(), Some("'"));
}

#[test]
fn api_01() {
    let data = "\
<e a:attr='a_ns' b:attr='b_ns' attr='no_ns' xmlns:b='http://www.ietf.org' \
    xmlns:a='http://www.w3.org' xmlns='http://www.uvic.ca'/>
";

    let doc = Document::parse(data).unwrap();
    let p = doc.root_element();

    assert_eq!(p.attribute("attr"), Some("no_ns"));
    assert_eq!(p.has_attribute("attr"), true);

    assert_eq!(p.attribute(("http://www.w3.org", "attr")), Some("a_ns"));
    assert_eq!(p.has_attribute(("http://www.w3.org", "attr")), true);

    assert_eq!(p.attribute("attr2"), None);
    assert_eq!(p.has_attribute("attr2"), false);

    assert_eq!(p.attribute(("http://www.w2.org", "attr")), None);
    assert_eq!(p.has_attribute(("http://www.w2.org", "attr")), false);

    assert_eq!(p.attribute("b"), None);
    assert_eq!(p.has_attribute("b"), false);

    assert_eq!(p.attribute("xmlns"), None);
    assert_eq!(p.has_attribute("xmlns"), false);
}

#[test]
fn get_pi() {
    let data = "\
<?target value?>
<root/>
";

    let doc = Document::parse(data).unwrap();
    let node = doc.root().first_child().unwrap();
    assert_eq!(node.pi(), Some(PI { target: "target", value: Some("value") }));
}

#[test]
fn lookup_prefix_01() {
    let data = "<e xmlns:n1='http://www.w3.org' n1:a='b1'/>";

    let doc = Document::parse(data).unwrap();
    let node = doc.root_element();
    assert_eq!(node.lookup_prefix("http://www.w3.org"), Some("n1"));
    assert_eq!(node.lookup_prefix("http://www.w4.org"), None);
}

#[test]
fn lookup_prefix_02() {
    let data = "<e xml:space='preserve'/>";

    let doc = Document::parse(data).unwrap();
    let node = doc.root_element();
    assert_eq!(node.lookup_prefix(NS_XML_URI), Some("xml"));
}

#[test]
fn lookup_namespace_uri() {
    let data = "<e xmlns:n1='http://www.w3.org' xmlns='http://www.w4.org'/>";

    let doc = Document::parse(data).unwrap();
    let node = doc.root_element();
    assert_eq!(node.lookup_namespace_uri("n1"), Some("http://www.w3.org"));
    assert_eq!(node.lookup_namespace_uri(""), Some("http://www.w4.org"));
    assert_eq!(node.lookup_namespace_uri("n2"), None);
}

#[test]
fn text_pos_01() {
    let data = "\
<e a='b'>
    <!-- comment -->
    <p>Text</p>
</e>
";

    let doc = Document::parse(data).unwrap();
    let node = doc.root_element();

    assert_eq!(node.node_pos(), TextPos::new(1, 1));
    assert_eq!(node.attribute_pos("a").unwrap(), TextPos::new(1, 4));
    assert_eq!(node.attribute_value_pos("a").unwrap(), TextPos::new(1, 7));

    // first child is a text/whitespace, not a comment
    let comm = node.first_child().unwrap().next_sibling().unwrap();
    assert_eq!(comm.node_pos(), TextPos::new(2, 5));

    let p = comm.next_sibling().unwrap().next_sibling().unwrap();
    assert_eq!(p.node_pos(), TextPos::new(3, 5));

    let text = p.first_child().unwrap();
    assert_eq!(text.node_pos(), TextPos::new(3, 8));
}

#[test]
fn text_pos_02() {
    let data = "<n1:e xmlns:n1='http://www.w3.org' n1:a='b'/>";

    let doc = Document::parse(data).unwrap();
    let node = doc.root_element();

    assert_eq!(node.node_pos(), TextPos::new(1, 1));
    assert_eq!(node.attribute_pos(("http://www.w3.org", "a")).unwrap(), TextPos::new(1, 36));
    assert_eq!(node.attribute_value_pos(("http://www.w3.org", "a")).unwrap(), TextPos::new(1, 42));
}

#[test]
fn text_pos_03() {
    let data = "\
<!-- comment -->
<e/>
";

    let doc = Document::parse(data).unwrap();
    let node = doc.root_element();

    assert_eq!(node.node_pos(), TextPos::new(2, 1));
}
