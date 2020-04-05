use bencher::Bencher;
use bencher::{benchmark_group, benchmark_main};


fn tiny_xmlparser(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("fonts.conf").unwrap();
    bencher.iter(|| {
        for t in xmlparser::Tokenizer::from(text.as_str()) {
            let _ = t.unwrap();
        }
    })
}

fn medium_xmlparser(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("medium.svg").unwrap();
    bencher.iter(|| {
        for t in xmlparser::Tokenizer::from(text.as_str()) {
            let _ = t.unwrap();
        }
    })
}

fn large_xmlparser(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("large.plist").unwrap();
    bencher.iter(|| {
        for t in xmlparser::Tokenizer::from(text.as_str()) {
            let _ = t.unwrap();
        }
    })
}


fn tiny_xmlrs(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("fonts.conf").unwrap();
    bencher.iter(|| {
        for event in xml::EventReader::new(text.as_bytes()) {
            let _ = event.unwrap();
        }
    })
}

fn medium_xmlrs(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("medium.svg").unwrap();
    bencher.iter(|| {
        for event in xml::EventReader::new(text.as_bytes()) {
            let _ = event.unwrap();
        }
    })
}

fn large_xmlrs(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("large.plist").unwrap();
    bencher.iter(|| {
        for event in xml::EventReader::new(text.as_bytes()) {
            let _ = event.unwrap();
        }
    })
}


fn parse_via_quick_xml(text: &str) {
    let mut r = quick_xml::Reader::from_str(text);
    r.check_comments(true);
    let mut buf = Vec::new();
    let mut ns_buf = Vec::new();
    loop {
        match r.read_namespaced_event(&mut buf, &mut ns_buf) {
            Ok((_, quick_xml::events::Event::Start(_))) |
            Ok((_, quick_xml::events::Event::Empty(_))) => (),
            Ok((_, quick_xml::events::Event::Text(ref e))) => {
                e.unescaped().unwrap();
                ()
            }
            Ok((_, quick_xml::events::Event::Eof)) => break,
            _ => (),
        }
        buf.clear();
    }
}

fn tiny_quick_xml(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("fonts.conf").unwrap();
    bencher.iter(|| parse_via_quick_xml(&text))
}

fn medium_quick_xml(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("medium.svg").unwrap();
    bencher.iter(|| parse_via_quick_xml(&text))
}

fn large_quick_xml(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("large.plist").unwrap();
    bencher.iter(|| parse_via_quick_xml(&text))
}


fn tiny_roxmltree(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("fonts.conf").unwrap();
    bencher.iter(|| roxmltree::Document::parse(&text).unwrap())
}

fn medium_roxmltree(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("medium.svg").unwrap();
    bencher.iter(|| roxmltree::Document::parse(&text).unwrap())
}

fn large_roxmltree(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("large.plist").unwrap();
    bencher.iter(|| roxmltree::Document::parse(&text).unwrap())
}


fn tiny_xmltree(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("fonts.conf").unwrap();
    bencher.iter(|| xmltree::Element::parse(text.as_bytes()).unwrap())
}

fn medium_xmltree(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("medium.svg").unwrap();
    bencher.iter(|| xmltree::Element::parse(text.as_bytes()).unwrap())
}

fn large_xmltree(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("large.plist").unwrap();
    bencher.iter(|| xmltree::Element::parse(text.as_bytes()).unwrap())
}


fn tiny_sdx_document(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("fonts.conf").unwrap();
    bencher.iter(|| sxd_document::parser::parse(&text).unwrap())
}

fn medium_sdx_document(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("medium.svg").unwrap();
    bencher.iter(|| sxd_document::parser::parse(&text).unwrap())
}

fn large_sdx_document(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("large.plist").unwrap();
    bencher.iter(|| sxd_document::parser::parse(&text).unwrap())
}


fn tiny_minidom(bencher: &mut Bencher) {
    let data = std::fs::read_to_string("fonts.conf").unwrap();
    bencher.iter(|| {
        let _root: minidom::Element = data.parse().unwrap();
    })
}

fn medium_minidom(bencher: &mut Bencher) {
    let data = std::fs::read_to_string("medium.svg").unwrap();
    bencher.iter(|| {
        let _root: minidom::Element = data.parse().unwrap();
    })
}

fn large_minidom(bencher: &mut Bencher) {
    let data = std::fs::read_to_string("large.plist").unwrap();
    bencher.iter(|| {
        let _root: minidom::Element = data.parse().unwrap();
    })
}

fn roxmltree_iter_descendants_inexpensive_matches(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("large.plist").unwrap();
    let doc = roxmltree::Document::parse(&text).unwrap();
    let root = doc.root();
    bencher.iter(|| {
        root.descendants().filter(|node| {
            node.tag_name().name() == "string"
        }).count();
    })
}

fn roxmltree_iter_descendants_expensive_matches(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("large.plist").unwrap();
    let doc = roxmltree::Document::parse(&text).unwrap();
    let root = doc.root();
    bencher.iter(|| {
        root.descendants().filter_map(|node| {
            if node.is_text() && node.text().unwrap().contains("twitter") {
                Some(node)
            } else {
                None
            }
        }).count();
    })
}

fn roxmltree_iter_children(bencher: &mut Bencher) {
    let text = std::fs::read_to_string("large.plist").unwrap();
    let doc = roxmltree::Document::parse(&text).unwrap();
    let root = doc.root();
    let large_array = root.descendants().find(|node| node.tag_name().name() == "array").unwrap();
    bencher.iter(|| {
        large_array.children().count();
    })
}

benchmark_group!(iter,
    roxmltree_iter_descendants_inexpensive_matches,
    roxmltree_iter_descendants_expensive_matches,
    roxmltree_iter_children);
benchmark_group!(roxmltree, tiny_roxmltree, medium_roxmltree, large_roxmltree);
benchmark_group!(xmltree, tiny_xmltree, medium_xmltree, large_xmltree);
benchmark_group!(sdx, tiny_sdx_document, medium_sdx_document, large_sdx_document);
benchmark_group!(minidom, tiny_minidom, medium_minidom, large_minidom);
benchmark_group!(xmlparser, tiny_xmlparser, medium_xmlparser, large_xmlparser);
benchmark_group!(xmlrs, tiny_xmlrs, medium_xmlrs, large_xmlrs);
benchmark_group!(quick_xml, tiny_quick_xml, medium_quick_xml, large_quick_xml);
benchmark_main!(roxmltree, xmltree, sdx, minidom, xmlparser, xmlrs, quick_xml, iter);
