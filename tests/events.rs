use roxmltree::{parse_events, Document, NodeType, ParsingOptions, ValidatedEvent};

type AttributeSnapshot = (Option<String>, String, String, bool, Vec<std::ops::Range<usize>>);

fn attribute_snapshot(attribute: roxmltree::Attribute<'_, '_>) -> AttributeSnapshot {
    #[cfg(feature = "positions")]
    let positions = vec![attribute.range(), attribute.range_qname(), attribute.range_value()];
    #[cfg(not(feature = "positions"))]
    let positions = Vec::new();
    (
        attribute.namespace().map(str::to_owned),
        attribute.name().to_owned(),
        attribute.value().to_owned(),
        matches!(attribute.value_storage(), roxmltree::StringStorage::Borrowed(_)),
        positions,
    )
}

#[derive(Debug, PartialEq, Eq)]
enum Snapshot {
    Start(
        u32,
        u32,
        Option<String>,
        String,
        Vec<AttributeSnapshot>,
        Vec<(Option<String>, String)>,
        usize,
    ),
    End(u32, usize),
    Text(u32, u32, String, std::ops::Range<usize>),
    Comment(u32, u32, String, std::ops::Range<usize>),
    Pi(u32, u32, String, Option<String>, std::ops::Range<usize>),
}

fn snapshot(event: ValidatedEvent<'_, '_>) -> Snapshot {
    match event {
        ValidatedEvent::ElementStart {
            id,
            parent,
            tag_name,
            attributes,
            namespaces,
            start,
        } => Snapshot::Start(
            id.get(),
            parent.get(),
            tag_name.namespace().map(str::to_owned),
            tag_name.name().to_owned(),
            attributes.map(attribute_snapshot).collect(),
            namespaces
                .map(|n| (n.name().map(str::to_owned), n.uri().to_owned()))
                .collect(),
            if cfg!(feature = "positions") { start } else { 0 },
        ),
        ValidatedEvent::ElementEnd { id, end } => {
            Snapshot::End(id.get(), if cfg!(feature = "positions") { end } else { 0 })
        }
        ValidatedEvent::Text {
            id,
            parent,
            text,
            range,
        } => Snapshot::Text(id.get(), parent.get(), text.to_owned(), position(range)),
        ValidatedEvent::Comment {
            id,
            parent,
            text,
            range,
        } => Snapshot::Comment(id.get(), parent.get(), text.to_owned(), position(range)),
        ValidatedEvent::ProcessingInstruction { id, parent, pi, range } => Snapshot::Pi(
            id.get(),
            parent.get(),
            pi.target.to_owned(),
            pi.value.map(str::to_owned),
            position(range),
        ),
    }
}

fn position(range: std::ops::Range<usize>) -> std::ops::Range<usize> {
    if cfg!(feature = "positions") {
        range
    } else {
        0..0
    }
}

fn indexed(doc: &Document<'_>) -> Vec<Snapshot> {
    fn children(node: roxmltree::Node<'_, '_>, events: &mut Vec<Snapshot>) {
        for node in node.children() {
            #[cfg(feature = "positions")]
            let range = node.range();
            #[cfg(not(feature = "positions"))]
            let range = 0..0;
            let id = node.id();
            let parent = node.parent().unwrap().id();
            events.push(snapshot(match node.node_type() {
                NodeType::Element => ValidatedEvent::ElementStart {
                    id,
                    parent,
                    tag_name: node.tag_name(),
                    attributes: node.attributes(),
                    namespaces: node.namespaces(),
                    start: range.start,
                },
                NodeType::Text => ValidatedEvent::Text {
                    id,
                    parent,
                    text: node.text().unwrap(),
                    range: range.clone(),
                },
                NodeType::Comment => ValidatedEvent::Comment {
                    id,
                    parent,
                    text: node.text().unwrap(),
                    range: range.clone(),
                },
                NodeType::PI => ValidatedEvent::ProcessingInstruction {
                    id,
                    parent,
                    pi: node.pi().unwrap(),
                    range: range.clone(),
                },
                NodeType::Root => unreachable!(),
            }));
            children(node, events);
            if node.is_element() {
                events.push(snapshot(ValidatedEvent::ElementEnd { id, end: range.end }));
            }
        }
    }
    let mut events = Vec::new();
    children(doc.root(), &mut events);
    events
}

fn parity(xml: &str, allow_dtd: bool, nodes_limit: u32) {
    let options = || ParsingOptions {
        allow_dtd,
        nodes_limit,
        ..ParsingOptions::default()
    };
    let expected = Document::parse_with_options(xml, options());
    let mut actual = Vec::new();
    let result = parse_events(xml, options(), |event| actual.push(snapshot(event)));
    match (expected, result) {
        (Ok(doc), Ok(())) => assert_eq!(actual, indexed(&doc), "{xml}"),
        (Err(expected), Err(actual)) => assert_eq!(actual, expected, "{xml}"),
        (expected, actual) => panic!("acceptance differs: {expected:?} / {actual:?}: {xml}"),
    }
}

#[test]
fn events_match_the_existing_xml_corpus() {
    let mut files = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/files"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "xml"))
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files.len(), 92);
    let mut non_utf8 = 0;
    for path in files {
        let bytes = std::fs::read(path).unwrap();
        match std::str::from_utf8(&bytes) {
            Ok(xml) => parity(xml, true, u32::MAX),
            Err(_) => non_utf8 += 1,
        }
    }
    assert_eq!(non_utf8, 1, "neither &str API accepts the non-UTF-8 fixture");
}

#[test]
fn semantic_errors_and_logical_text_nodes_match() {
    let fixtures = [
        "",
        "<!--empty-->",
        "<a></b>",
        "<a>",
        "<a/><b/>",
        "<p:a/>",
        "<a p:x='1'/>",
        "<a xmlns:xml='wrong'/>",
        "<a xmlns:p='urn:u' xmlns:q='urn:u' p:x='1' q:x='2'/>",
        "<a xmlns:p='u' xmlns:p='v'/>",
        "<a xmlns='http://www.w3.org/2000/xmlns/'/>",
        "<!DOCTYPE a [<!ENTITY x '<b>'>]><a>&x;</a>",
        "<!DOCTYPE a [<!ENTITY x '<b></b></a>'>]><a>&x;",
        "<!DOCTYPE a [<!ENTITY x '<b/>'>]><a>&x;&x;</a>",
        "<!DOCTYPE a [<!ENTITY x 'one<![CDATA[two]]>three'>]><a>&x;</a>",
        "<a>&unknown;</a>",
        "<a>&malformed</a>",
        "<a x='&unknown;'/>",
        "<!DOCTYPE a [<!ENTITY x '&x;'>]><a>&x;</a>",
        "<!DOCTYPE a [<!ENTITY x '</a>'>]><a>&x;</a>",
        "<!DOCTYPE a [<!ENTITY x '<p/>'>]><a>before&x;after</a>",
        "<!DOCTYPE a [<!ENTITY x 'one&#xD;two'>]><a>A&x;B<![CDATA[C]]>D</a>",
        "<!DOCTYPE a [<!ENTITY x '&#60;'>]><a x='&x;'/>",
        "<a x='a\r\nb\tc&#xA;d'>one\r\ntwo<![CDATA[three\r\nfour]]>five</a>",
        "<?pi before?><a xmlns='u' xmlns:p='v'><p:b p:c='d'/><!--x--><?pi value?></a>",
    ];
    for xml in fixtures {
        for allow_dtd in [false, true] {
            for limit in [0, 1, 2, 3, 4, 8, u32::MAX] {
                parity(xml, allow_dtd, limit);
            }
        }
    }
}

#[test]
fn external_entity_resolution_and_errors_match() {
    let xml = "<!DOCTYPE a [<!ENTITY one SYSTEM 'one'><!ENTITY two PUBLIC 'public' 'two'>]><a>&one;&two;</a>";
    for fail in [false, true] {
        let run = |events: bool| {
            let calls = std::cell::RefCell::new(Vec::new());
            let resolver = |public: Option<&str>, uri: &str| {
                calls.borrow_mut().push((public.map(str::to_owned), uri.to_owned()));
                if fail {
                    Err("resolver rejected".to_owned())
                } else {
                    Ok(Some("<b/>text"))
                }
            };
            let options = ParsingOptions {
                allow_dtd: true,
                entity_resolver: Some(&resolver),
                ..ParsingOptions::default()
            };
            let result = if events {
                let mut captured = Vec::new();
                parse_events(xml, options, |event| captured.push(snapshot(event))).map(|()| captured)
            } else {
                Document::parse_with_options(xml, options).map(|doc| indexed(&doc))
            };
            (result, calls.into_inner())
        };
        assert_eq!(run(true), run(false));
    }
}

#[test]
fn completed_namespace_scopes_still_count_towards_the_same_limit() {
    let mut xml = String::from("<root>");
    for number in 0..65_536 {
        xml.push_str(&format!("<e xmlns:p='urn:{number}'/>"));
    }
    xml.push_str("</root>");
    parity(&xml, true, u32::MAX);
    assert_eq!(
        parse_events(&xml, ParsingOptions::default(), |_| {}),
        Err(roxmltree::Error::NamespacesLimitReached)
    );
}
