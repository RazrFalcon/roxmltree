extern crate roxmltree;
extern crate rustc_test;
#[macro_use] extern crate pretty_assertions;

use roxmltree::*;

use rustc_test::{TestDesc, TestDescAndFn, DynTestName, DynTestFn};

use std::env;
use std::path;
use std::fs;
use std::io::Read;
use std::fmt::Write;
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
struct TStr<'a>(pub &'a str);

impl<'a> fmt::Debug for TStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


trait HasExtension {
    fn has_extension(&self, ext: &str) -> bool;
}

impl HasExtension for path::Path {
    fn has_extension(&self, ext: &str) -> bool {
        if let Some(e) = self.extension() { e == ext } else { false }
    }
}


// List of not yet supported test cases.
static IGNORE: &[&str] = &[
    "entity_007.xml",
    "tree_003.xml",
];


#[test]
fn run() {
    let mut tests = Vec::new();

    for entry in fs::read_dir("tests/files").unwrap() {
        let entry = entry.unwrap();

        if !entry.path().has_extension("xml") {
            continue;
        }

        let file_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
        if !IGNORE.contains(&file_name.as_str()) {
            tests.push(create_test(entry.path()));
        }
    }

    let mut args: Vec<String> = env::args().collect();

    if let Some(idx) = args.iter().position(|x| *x == "--nocapture") {
        args.remove(idx);
    }

    rustc_test::test_main(&args, tests);
}

fn create_test(path: path::PathBuf) -> TestDescAndFn {
    let name = path.file_name().unwrap().to_str().unwrap().to_string();

    TestDescAndFn {
        desc: TestDesc::new(DynTestName(name)),
        testfn: DynTestFn(Box::new(move || actual_test(path.clone()))),
    }
}

fn actual_test(path: path::PathBuf) {
    let expected = load_file(&path.with_extension("yaml"));

    let input_xml = load_file(&path);
    let doc = match Document::parse(&input_xml) {
        Ok(v) => v,
        Err(e) => {
            assert_eq!(TStr(&format!("error: \"{}\"", e)), TStr(expected.trim()));
            return;
        }
    };

    assert_eq!(TStr(&expected), TStr(&to_yaml(&doc)));
}

fn load_file(path: &path::Path) -> String {
    let mut file = fs::File::open(&path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}

fn to_yaml(doc: &Document) -> String {
    let mut s = String::new();
    _to_yaml(doc, &mut s).unwrap();
    s
}

fn _to_yaml(doc: &Document, s: &mut String) -> Result<(), fmt::Error> {
    if !doc.root().has_children() {
        return write!(s, "Document:");
    }

    macro_rules! writeln_indented {
        ($depth:expr, $f:expr, $fmt:expr) => {
            for _ in 0..$depth { write!($f, "  ")?; }
            writeln!($f, $fmt)?;
        };
        ($depth:expr, $f:expr, $fmt:expr, $($arg:tt)*) => {
            for _ in 0..$depth { write!($f, "  ")?; }
            writeln!($f, $fmt, $($arg)*)?;
        };
    }

    fn print_children(parent: Node, depth: usize, s: &mut String) -> Result<(), fmt::Error> {
        for child in parent.children() {
            match child.node_type() {
                NodeType::Element => {
                    writeln_indented!(depth, s, "- Element:");

                    if child.tag_name().has_namespace() {
                        writeln_indented!(depth + 2, s, "tag_name: {}@{}",
                                          child.tag_name().name(), child.tag_name().namespace());
                    } else {
                        writeln_indented!(depth + 2, s, "tag_name: {}", child.tag_name().name());
                    }

                    if !child.attributes().is_empty() {
                        let mut attrs = Vec::new();
                        for attr in child.attributes() {
                            if !attr.namespace().is_empty() {
                                attrs.push((format!("{}@{}", attr.name(), attr.namespace()), attr.value()));
                            } else {
                                attrs.push((attr.name().to_string(), attr.value()));
                            }
                        }
                        attrs.sort_by(|a, b| a.0.cmp(&b.0));

                        writeln_indented!(depth + 2, s, "attributes:");
                        for (name, value) in attrs {
                            writeln_indented!(depth + 3, s, "{}: {:?}", name, value);
                        }
                    }

                    if !child.namespaces().is_empty() {
                        let mut ns_list = Vec::new();
                        for ns in child.namespaces() {
                            let name = if ns.name().is_empty() { "\"\"" } else { ns.name() };
                            let uri = if ns.uri().is_empty() { "\"\"" } else { ns.uri() };
                            ns_list.push((name, uri));
                        }
                        ns_list.sort_by(|a, b| a.0.cmp(&b.0));

                        writeln_indented!(depth + 2, s, "namespaces:");
                        for (name, uri) in ns_list {
                            writeln_indented!(depth + 3, s, "{}: {}", name, uri);
                        }
                    }

                    if child.has_children() {
                        writeln_indented!(depth + 2, s, "children:");
                        print_children(child, depth + 3, s)?;
                    }
                }
                NodeType::Text => {
                    writeln_indented!(depth, s, "- Text: {:?}", child.text().unwrap());
                }
                NodeType::Comment => {
                    writeln_indented!(depth, s, "- Comment: {:?}", child.text().unwrap());
                }
                NodeType::PI => {
                    if child.parent().unwrap().is_root() {
                        continue;
                    }

                    writeln_indented!(depth, s, "- PI:");

                    let pi = child.pi().unwrap();
                    writeln_indented!(depth + 2, s, "target: {:?}", pi.target);
                    if let Some(value) = pi.value {
                        writeln_indented!(depth + 2, s, "value: {:?}", value);
                    }
                }
                NodeType::Root => {}
            }
        }

        Ok(())
    }

    writeln!(s, "Document:")?;
    print_children(doc.root(), 1, s)?;

    Ok(())
}
