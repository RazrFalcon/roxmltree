/*!
Represent an [XML 1.0](https://www.w3.org/TR/xml/) document as a read-only tree.

The root point of the documentations is [`Document::parse`].

You can find more details in the [README] and [parsing doc].

The tree structure itself is a heavily modified <https://github.com/programble/ego-tree>
License: ISC.

[`Document::parse`]: struct.Document.html#method.parse
[README]: https://github.com/RazrFalcon/roxmltree/blob/master/README.md
[parsing doc]: https://github.com/RazrFalcon/roxmltree/blob/master/docs/parsing.md
*/

#![cfg_attr(feature = "cargo-clippy", allow(collapsible_if))]

#![doc(html_root_url = "https://docs.rs/roxmltree/0.1.0")]

#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate xmlparser;

use std::fmt;
use std::ops::{Deref, Range};
use std::rc::Rc;

pub use xmlparser::TextPos;

mod parse;
pub use parse::*;


/// The <http://www.w3.org/XML/1998/namespace> URI.
pub const NS_XML_URI: &str = "http://www.w3.org/XML/1998/namespace";

/// The <http://www.w3.org/2000/xmlns/> URI.
pub const NS_XMLNS_URI: &str = "http://www.w3.org/2000/xmlns/";


/// An XML tree container.
///
/// A tree consists of [`Nodes`].
/// There are no separate structs for each node type.
/// So you should check the current node type yourself via [`Node::node_type()`].
/// There are only [5 types](enum.NodeType.html):
/// Root, Element, PI, Comment and Text.
///
/// As you can see there are no XML declaration and CDATA types.
/// The XML declaration is basically skipped, since it doesn't contains any
/// valuable information (we support only UTF-8 anyway).
/// And CDATA will be converted into a Text node as is, without
/// any preprocessing (you can read more about it
/// [here](https://github.com/RazrFalcon/roxmltree/blob/master/docs/parsing.md)).
///
/// Also, the Text node data can be accesses from the text node itself or from
/// the parent element via [`Node::text()`] or [`Node::tail()`].
///
/// [`Nodes`]: struct.Node.html
/// [`Node::node_type()`]: struct.Node.html#method.node_type
/// [`Node::text()`]: struct.Node.html#method.text
/// [`Node::tail()`]: struct.Node.html#method.tail
#[derive(PartialEq)]
pub struct Document<'d> {
    /// An original data.
    ///
    /// Required for `text_pos` methods.
    text: &'d str,
    nodes: Vec<NodeData<'d>>,
    attrs: Vec<Attribute<'d>>,
    namespaces: Namespaces<'d>,
}

impl<'d> Document<'d> {
    /// Returns the root node.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e/>").unwrap();
    /// assert!(doc.root().is_root());
    /// assert!(doc.root().first_child().unwrap().has_tag_name("e"));
    /// ```
    pub fn root(&self) -> Node {
        Node { id: NodeId(0), d: &self.nodes[0], doc: self }
    }

    /// Returns the root element of the document.
    ///
    /// Unlike `root`, will return a first element node.
    ///
    /// The root element is always exists.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<!-- comment --><e/>").unwrap();
    /// assert!(doc.root_element().has_tag_name("e"));
    /// ```
    pub fn root_element(&self) -> Node {
        // `unwrap` is safe, because the `Document` is guarantee to have at least one element.
        self.root().first_element_child().unwrap()
    }

    /// Returns an iterator over document's descendant nodes.
    ///
    /// Shorthand for `doc.root().descendants()`.
    pub fn descendants(&self) -> Descendants {
        self.root().descendants()
    }

    /// Calculates `TextPos` in the original document from position in bytes.
    ///
    /// **Note:** this operation is expensive.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("\
    /// <!-- comment -->
    /// <e/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.text_pos_from(10), roxmltree::TextPos::new(1, 11));
    /// assert_eq!(doc.text_pos_from(9999), roxmltree::TextPos::new(2, 5));
    /// ```
    pub fn text_pos_from(&self, pos: usize) -> TextPos {
        xmlparser::Stream::from(self.text).gen_error_pos_from(pos)
    }
}

impl<'d> fmt::Debug for Document<'d> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if !self.root().has_children() {
            return write!(f, "Document []");
        }

        macro_rules! writeln_indented {
            ($depth:expr, $f:expr, $fmt:expr) => {
                for _ in 0..$depth { write!($f, "    ")?; }
                writeln!($f, $fmt)?;
            };
            ($depth:expr, $f:expr, $fmt:expr, $($arg:tt)*) => {
                for _ in 0..$depth { write!($f, "    ")?; }
                writeln!($f, $fmt, $($arg)*)?;
            };
        }

        fn print_vec<T: fmt::Debug>(prefix: &str, data: &[T], depth: usize, f: &mut fmt::Formatter)
            -> Result<(), fmt::Error>
        {
            if data.is_empty() {
                return Ok(());
            }

            writeln_indented!(depth, f, "{}: [", prefix);
            for v in data {
                writeln_indented!(depth + 1, f, "{:?}", v);
            }
            writeln_indented!(depth, f, "]");

            Ok(())
        }

        fn print_children(parent: Node, depth: usize, f: &mut fmt::Formatter)
            -> Result<(), fmt::Error>
        {
            for child in parent.children() {
                if child.is_element() {
                    writeln_indented!(depth, f, "Element {{");
                    writeln_indented!(depth, f, "    tag_name: {:?}", child.tag_name());
                    print_vec("attributes", child.attributes(), depth + 1, f)?;
                    print_vec("namespaces", child.namespaces(), depth + 1, f)?;

                    if child.has_children() {
                        writeln_indented!(depth, f, "    children: [");
                        print_children(child, depth + 2, f)?;
                        writeln_indented!(depth, f, "    ]");
                    }

                    writeln_indented!(depth, f, "}}");
                } else {
                    writeln_indented!(depth, f, "{:?}", child);
                }
            }

            Ok(())
        }

        writeln!(f, "Document [")?;
        print_children(self.root(), 1, f)?;
        writeln!(f, "]")?;

        Ok(())
    }
}


/// List of supported node types.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NodeType {
    /// The root node of the `Document`.
    Root,
    /// An element node.
    ///
    /// Only an element can have tag name and attributes.
    Element,
    /// A processing instruction.
    PI,
    /// A comment node.
    Comment,
    /// A text node.
    Text,
}

/// A processing instruction.
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct PI<'d> {
    pub target: &'d str,
    pub value: Option<&'d str>,
}


/// Node ID.
///
/// Index into a `Tree`-internal `Vec`.
#[derive(Clone, Copy, PartialEq)]
struct NodeId(usize);


#[derive(PartialEq)]
enum NodeKind<'d> {
    Root,
    Element {
        tag_name: ExpandedNameOwned<'d>,
        attributes: Range<usize>,
        namespaces: Range<usize>,
    },
    PI(PI<'d>),
    Comment(&'d str),
    Text(String),
}


#[derive(PartialEq)]
struct NodeData<'d> {
    parent: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    children: Option<(NodeId, NodeId)>,
    kind: NodeKind<'d>,
    orig_pos: usize,
}


/// An attribute.
#[derive(PartialEq)]
pub struct Attribute<'d> {
    name: ExpandedNameOwned<'d>,
    value: String,
    attr_pos: usize,
    value_pos: usize,
}

impl<'d> Attribute<'d> {
    /// Returns attribute's namespace URI.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org' a='b' n:a='c'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().attributes()[0].namespace(), "");
    /// assert_eq!(doc.root_element().attributes()[1].namespace(), "http://www.w3.org");
    /// ```
    pub fn namespace(&self) -> &str {
        self.name.ns.as_str()
    }

    /// Returns attribute's name.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org' a='b' n:a='c'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().attributes()[0].name(), "a");
    /// assert_eq!(doc.root_element().attributes()[1].name(), "a");
    /// ```
    pub fn name(&self) -> &str {
        self.name.name
    }

    /// Returns attribute's value.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org' a='b' n:a='c'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().attributes()[0].value(), "b");
    /// assert_eq!(doc.root_element().attributes()[1].value(), "c");
    /// ```
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns attribute's name position in bytes in the original document.
    ///
    /// You can calculate a human-readable text position via [Node::text_pos_from].
    ///
    /// ```text
    /// <e attr='value'/>
    ///    ^
    /// ```
    ///
    /// [Node::text_pos_from]: struct.Node.html#method.text_pos_from
    pub fn pos(&self) -> usize {
        self.attr_pos
    }

    /// Returns attribute's value position in bytes in the original document.
    ///
    /// You can calculate a human-readable text position via [Node::text_pos_from].
    ///
    /// ```text
    /// <e attr='value'/>
    ///          ^
    /// ```
    ///
    /// [Node::text_pos_from]: struct.Node.html#method.text_pos_from
    pub fn value_pos(&self) -> usize {
        self.value_pos
    }
}

impl<'d> fmt::Debug for Attribute<'d> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Attribute {{ name: {:?}, value: {:?} }}",
               self.name, self.value)
    }
}


/// A namespace.
///
/// Contains URI and *prefix* pair.
#[derive(Clone, PartialEq, Debug)]
pub struct Namespace<'d> {
    name: &'d str,
    uri: Uri,
}

impl<'d> Namespace<'d> {
    /// Returns namespace name/prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().namespaces()[0].name(), "n");
    /// ```
    pub fn name(&self) -> &str {
        self.name
    }

    /// Returns namespace URI.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().namespaces()[0].uri(), "http://www.w3.org");
    /// ```
    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }
}


#[derive(PartialEq)]
struct Namespaces<'d>(Vec<Namespace<'d>>);

impl<'d> Namespaces<'d> {
    fn push_ns(&mut self, name: &'d str, uri: String) {
        self.0.push(Namespace {
            name,
            uri: Uri::new(uri),
        });
    }

    fn null_uri(&self) -> Uri {
        self[0].uri.clone()
    }

    fn xml_uri(&self) -> Uri {
        self[1].uri.clone()
    }

    fn get_by_prefix(&self, range: Range<usize>, prefix: &str) -> Uri {
        self[range].iter()
                   .find(|ns| ns.name == prefix)
                   .map(|ns| ns.uri.clone())
                   .unwrap_or_else(|| self.null_uri())
    }

    fn exists(&self, start: usize, prefix: &str) -> bool {
        self[start..].iter().any(|ns| ns.name == prefix)
    }
}

impl<'d> Deref for Namespaces<'d> {
    type Target = Vec<Namespace<'d>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


struct Uri(Rc<String>);

impl Uri {
    fn new(text: String) -> Self {
        Uri(Rc::new(text))
    }

    fn as_str(&self) -> &str{
        self.0.as_str()
    }
}

impl Clone for Uri {
    fn clone(&self) -> Self {
        Uri(Rc::clone(&self.0))
    }
}

impl PartialEq for Uri {
    fn eq(&self, other: &Uri) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl fmt::Debug for Uri {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}


#[derive(Clone, PartialEq)]
struct ExpandedNameOwned<'d> {
    ns: Uri,
    name: &'d str,
}

impl<'d> ExpandedNameOwned<'d> {
    fn as_ref(&self) -> ExpandedName {
        ExpandedName { ns: self.ns.as_str(), name: self.name }
    }

    fn has_namespace(&self) -> bool {
        !self.ns.as_str().is_empty()
    }
}

impl<'d> fmt::Debug for ExpandedNameOwned<'d> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.has_namespace() {
            write!(f, "{{{}}}{}", self.ns.as_str(), self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}


/// An expanded name.
///
/// Contains an namespace URI and name pair.
#[derive(Clone, Copy, PartialEq)]
pub struct ExpandedName<'d> {
    ns: &'d str,
    name: &'d str,
}

impl<'d> ExpandedName<'d> {
    /// Returns a namespace URI.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().tag_name().namespace(), "http://www.w3.org");
    /// ```
    pub fn namespace(&self) -> &str {
        self.ns
    }

    /// Checks that expanded name has a namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().tag_name().has_namespace(), true);
    /// ```
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns:n='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().tag_name().has_namespace(), false);
    /// ```
    pub fn has_namespace(&self) -> bool {
        !self.ns.is_empty()
    }

    /// Returns a name.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().tag_name().name(), "e");
    /// ```
    pub fn name(&self) -> &str {
        self.name
    }
}

impl<'d> fmt::Debug for ExpandedName<'d> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.has_namespace() {
            write!(f, "{{{}}}{}", self.ns, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl<'d> From<&'d str> for ExpandedName<'d> {
    fn from(v: &'d str) -> Self {
        ExpandedName {
            ns: "",
            name: v,
        }
    }
}

impl<'d> From<(&'d str, &'d str)> for ExpandedName<'d> {
    fn from(v: (&'d str, &'d str)) -> Self {
        ExpandedName {
            ns: v.0,
            name: v.1,
        }
    }
}


/// A node.
pub struct Node<'a, 'd: 'a> {
    /// Node ID.
    id: NodeId,

    /// Tree containing the node.
    doc: &'a Document<'d>,

    d: &'a NodeData<'d>,
}

impl<'a, 'd> Copy for Node<'a, 'd> {}

impl<'a, 'd> Clone for Node<'a, 'd> {
    fn clone(&self) -> Self { *self }
}

impl<'a, 'd> Eq for Node<'a, 'd> {}

impl<'a, 'd> PartialEq for Node<'a, 'd> {
    fn eq(&self, other: &Self) -> bool {
           self.id == other.id
        && self.doc as *const _ == other.doc as *const _
        && self.d as *const _ == other.d as *const _
    }
}

impl<'a, 'd: 'a> Node<'a, 'd> {
    /// Returns node's type.
    pub fn node_type(&self) -> NodeType {
        match self.d.kind {
            NodeKind::Root => NodeType::Root,
            NodeKind::Element { .. } => NodeType::Element,
            NodeKind::PI { .. } => NodeType::PI,
            NodeKind::Comment(_) => NodeType::Comment,
            NodeKind::Text(_) => NodeType::Text,
        }
    }

    /// Checks that node is a root node.
    pub fn is_root(&self) -> bool {
        self.node_type() == NodeType::Root
    }

    /// Checks that node is an element node.
    pub fn is_element(&self) -> bool {
        self.node_type() == NodeType::Element
    }

    /// Checks that node is a processing instruction node.
    pub fn is_pi(&self) -> bool {
        self.node_type() == NodeType::PI
    }

    /// Checks that node is a comment node.
    pub fn is_comment(&self) -> bool {
        self.node_type() == NodeType::Comment
    }

    /// Checks that node is a text node.
    pub fn is_text(&self) -> bool {
        self.node_type() == NodeType::Text
    }

    /// Returns node's document.
    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// Returns node's tag name.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().tag_name().namespace(), "http://www.w3.org");
    /// assert_eq!(doc.root_element().tag_name().name(), "e");
    /// ```
    pub fn tag_name(&'a self) -> ExpandedName<'a> {
        match self.d.kind {
            NodeKind::Element { ref tag_name, .. } => tag_name.as_ref(),
            _ => ExpandedName { ns: "", name: "" },
        }
    }

    /// Checks that node has a specified tag name.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns='http://www.w3.org'/>").unwrap();
    ///
    /// assert!(doc.root_element().has_tag_name("e"));
    /// assert!(doc.root_element().has_tag_name(("http://www.w3.org", "e")));
    ///
    /// assert!(!doc.root_element().has_tag_name("b"));
    /// assert!(!doc.root_element().has_tag_name(("http://www.w4.org", "e")));
    /// ```
    pub fn has_tag_name<N>(&self, name: N) -> bool
        where N: Into<ExpandedName<'a>>
    {
        let name = name.into();

        match self.d.kind {
            NodeKind::Element { ref tag_name, .. } => {
                if name.has_namespace() {
                    tag_name.as_ref() == name
                } else {
                    tag_name.name == name.name
                }
            }
            _ => false,
        }
    }

    /// Returns node's default namespace URI.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().default_namespace(), Some("http://www.w3.org"));
    /// ```
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns:n='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().default_namespace(), None);
    /// ```
    pub fn default_namespace(&self) -> Option<&str> {
        self.namespaces().iter().find(|ns| ns.name.is_empty()).map(|v| v.uri.as_str())
    }

    /// Returns element's namespace prefix.
    ///
    /// Returns an empty prefix:
    /// - if the current node is not an element
    /// - if the current element has a default namespace
    /// - if the current element has no namespace
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<n:e xmlns:n='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().resolve_tag_name_prefix(), "n");
    /// ```
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns:n='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().resolve_tag_name_prefix(), "");
    /// ```
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().resolve_tag_name_prefix(), "");
    /// ```
    pub fn resolve_tag_name_prefix(&self) -> &str {
        if !self.is_element() {
            return "";
        }

        let tag_ns = self.tag_name().ns;

        // Check for a default namespace first.
        if self.default_namespace() == Some(&tag_ns) {
            return "";
        }

        self.namespaces().iter().find(|ns| ns.uri.as_str() == tag_ns).map(|v| v.name).unwrap_or("")
    }

    /// Returns a prefix for a given namespace URI.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns:n='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().lookup_prefix("http://www.w3.org"), Some("n"));
    /// ```
    pub fn lookup_prefix(&self, uri: &str) -> Option<&str> {
        if uri == NS_XML_URI {
            return Some("xml");
        }

        self.namespaces().iter().find(|ns| ns.uri.as_str() == uri).map(|v| v.name)
    }

    /// Returns an URI for a given prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns:n='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().lookup_namespace_uri("n"), Some("http://www.w3.org"));
    /// ```
    pub fn lookup_namespace_uri(&self, prefix: &str) -> Option<&str> {
        self.namespaces().iter().find(|ns| ns.name == prefix).map(|v| v.uri.as_str())
    }

    /// Returns element's attribute value.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e a='b'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().attribute("a"), Some("b"));
    /// ```
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org' a='b' n:a='c'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().attribute("a"), Some("b"));
    /// assert_eq!(doc.root_element().attribute(("http://www.w3.org", "a")), Some("c"));
    /// ```
    pub fn attribute<N>(&self, name: N) -> Option<&str>
        where N: Into<ExpandedName<'a>>
    {
        let name = name.into();
        self.attributes().iter().find(|a| a.name.as_ref() == name).map(|a| a.value.as_ref())
    }

    /// Checks that element has a specified attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org' a='b' n:a='c'/>"
    /// ).unwrap();
    ///
    /// assert!(doc.root_element().has_attribute("a"));
    /// assert!(doc.root_element().has_attribute(("http://www.w3.org", "a")));
    ///
    /// assert!(!doc.root_element().has_attribute("b"));
    /// assert!(!doc.root_element().has_attribute(("http://www.w4.org", "a")));
    /// ```
    pub fn has_attribute<N>(&self, name: N) -> bool
        where N: Into<ExpandedName<'a>>
    {
        let name = name.into();
        self.attributes().iter().any(|a| a.name.as_ref() == name)
    }

    /// Returns element's attributes.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org' a='b' n:a='c'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().attributes().len(), 2);
    /// ```
    pub fn attributes(&self) -> &[Attribute] {
        match self.d.kind {
            NodeKind::Element { ref attributes, .. } => &self.doc.attrs[attributes.clone()],
            _ => &[],
        }
    }

    /// Calculates attribute's position in the original document.
    ///
    /// **Note:** this operation is expensive.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org' a='b' n:a='c'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().attribute_pos("a"),
    ///            Some(roxmltree::TextPos::new(1, 32)));
    /// assert_eq!(doc.root_element().attribute_pos(("http://www.w3.org", "a")),
    ///            Some(roxmltree::TextPos::new(1, 38)));
    /// ```
    pub fn attribute_pos<N>(&self, name: N) -> Option<TextPos>
        where N: Into<ExpandedName<'a>>
    {
        let name = name.into();
        self.attributes().iter().find(|a| a.name.as_ref() == name)
            .map(|a| self.document().text_pos_from(a.attr_pos))
    }

    /// Calculates attribute's value position in the original document.
    ///
    /// **Note:** this operation is expensive.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org' a='b' n:a='c'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().attribute_value_pos("a"),
    ///            Some(roxmltree::TextPos::new(1, 35)));
    /// assert_eq!(doc.root_element().attribute_value_pos(("http://www.w3.org", "a")),
    ///            Some(roxmltree::TextPos::new(1, 43)));
    /// ```
    pub fn attribute_value_pos<N>(&self, name: N) -> Option<TextPos>
        where N: Into<ExpandedName<'a>>
    {
        let name = name.into();
        self.attributes().iter().find(|a| a.name.as_ref() == name)
            .map(|a| self.document().text_pos_from(a.value_pos))
    }

    /// Returns element's namespaces.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().namespaces().len(), 1);
    /// ```
    pub fn namespaces(&self) -> &[Namespace] {
        match self.d.kind {
            NodeKind::Element { ref namespaces, .. } => {
                &self.doc.namespaces[namespaces.clone()]
            }
            _ => &[],
        }
    }

    /// Returns node's text.
    ///
    /// - for an element will return a first text child
    /// - for a comment will return a self text
    /// - for a text node will return a self text
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("\
    /// <p>
    ///     text
    /// </p>
    /// ").unwrap();
    ///
    /// assert_eq!(doc.root_element().text(),
    ///            Some("\n    text\n"));
    /// assert_eq!(doc.root_element().first_child().unwrap().text(),
    ///            Some("\n    text\n"));
    /// ```
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<!-- comment --><e/>").unwrap();
    ///
    /// assert_eq!(doc.root().first_child().unwrap().text(), Some(" comment "));
    /// ```
    pub fn text(&self) -> Option<&str> {
        match self.d.kind {
            NodeKind::Element { .. } => {
                match self.first_child() {
                    Some(child) if child.is_text() => {
                        match self.doc.nodes[child.id.0].kind {
                            NodeKind::Text(ref text) => Some(text),
                            _ => None
                        }
                    }
                    _ => None,
                }
            }
            NodeKind::Comment(text) => Some(text),
            NodeKind::Text(ref text) => Some(text),
            _ => None,
        }
    }

    /// Returns element's tail text.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("\
    /// <root>
    ///     text1
    ///     <p/>
    ///     text2
    /// </root>
    /// ").unwrap();
    ///
    /// let p = doc.descendants().find(|n| n.has_tag_name("p")).unwrap();
    /// assert_eq!(p.tail(), Some("\n    text2\n"));
    /// ```
    pub fn tail(&self) -> Option<&str> {
        if !self.is_element() {
            return None;
        }

        match self.next_sibling().map(|n| n.id) {
            Some(id) => {
                match self.doc.nodes[id.0].kind {
                    NodeKind::Text(ref text) => Some(text),
                    _ => None
                }
            }
            None => None,
        }
    }

    /// Returns node as Processing Instruction.
    pub fn pi(&self) -> Option<PI> {
        match self.d.kind {
            NodeKind::PI(pi) => Some(pi),
            _ => None,
        }
    }

    fn gen_node(&self, id: NodeId) -> Node<'a, 'd> {
        Node { id, d: &self.doc.nodes[id.0], doc: self.doc }
    }

    /// Returns the parent of this node.
    pub fn parent(&self) -> Option<Self> {
        self.d.parent.map(|id| self.gen_node(id))
    }

    /// Returns the parent element of this node.
    pub fn parent_element(&self) -> Option<Self> {
        self.ancestors().filter(|n| n.is_element()).nth(0)
    }

    /// Returns the previous sibling of this node.
    pub fn prev_sibling(&self) -> Option<Self> {
        self.d.prev_sibling.map(|id| self.gen_node(id))
    }

    /// Returns the next sibling of this node.
    pub fn next_sibling(&self) -> Option<Self> {
        self.d.next_sibling.map(|id| self.gen_node(id))
    }

    /// Returns the first child of this node.
    pub fn first_child(&self) -> Option<Self> {
        self.d.children.map(|(id, _)| self.gen_node(id))
    }

    /// Returns the first element child of this node.
    pub fn first_element_child(&self) -> Option<Self> {
        self.children().filter(|n| n.is_element()).nth(0)
    }

    /// Returns the last child of this node.
    pub fn last_child(&self) -> Option<Self> {
        self.d.children.map(|(_, id)| self.gen_node(id))
    }

    /// Returns the last element child of this node.
    pub fn last_element_child(&self) -> Option<Self> {
        self.children().filter(|n| n.is_element()).last()
    }

    /// Returns true if this node has siblings.
    pub fn has_siblings(&self) -> bool {
        self.d.prev_sibling.is_some() || self.d.next_sibling.is_some()
    }

    /// Returns true if this node has children.
    pub fn has_children(&self) -> bool {
        self.d.children.is_some()
    }

    /// Returns an iterator over ancestor nodes.
    pub fn ancestors(&self) -> Ancestors<'a, 'd> {
        Ancestors(self.parent())
    }

    /// Returns an iterator over previous sibling nodes.
    pub fn prev_siblings(&self) -> PrevSiblings<'a, 'd> {
        PrevSiblings(self.prev_sibling())
    }

    /// Returns an iterator over next sibling nodes.
    pub fn next_siblings(&self) -> NextSiblings<'a, 'd> {
        NextSiblings(self.next_sibling())
    }

    /// Returns an iterator over first children nodes.
    pub fn first_children(&self) -> FirstChildren<'a, 'd> {
        FirstChildren(self.first_child())
    }

    /// Returns an iterator over last children nodes.
    pub fn last_children(&self) -> LastChildren<'a, 'd> {
        LastChildren(self.last_child())
    }

    /// Returns an iterator over children nodes.
    pub fn children(&self) -> Children<'a, 'd> {
        Children { front: self.first_child(), back: self.last_child() }
    }

    /// Returns an iterator which traverses the subtree starting at this node.
    pub fn traverse(&self) -> Traverse<'a, 'd> {
        Traverse { root: *self, edge: None }
    }

    /// Returns an iterator over this node and its descendants.
    pub fn descendants(&self) -> Descendants<'a, 'd> {
        Descendants(self.traverse())
    }

    /// Returns node's position in bytes in the original document.
    pub fn pos(&self) -> usize {
        self.d.orig_pos
    }

    /// Calculates node's position in the original document.
    ///
    /// **Note:** this operation is expensive.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("\
    /// <!-- comment -->
    /// <e/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().node_pos(), roxmltree::TextPos::new(2, 1));
    /// ```
    pub fn node_pos(&self) -> TextPos {
        self.document().text_pos_from(self.d.orig_pos)
    }
}

impl<'a, 'd: 'a> fmt::Debug for Node<'a, 'd> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.d.kind {
            NodeKind::Root => write!(f, "Root"),
            NodeKind::Element { .. } => {
                write!(f, "Element {{ tag_name: {:?}, attributes: {:?}, namespaces: {:?} }}",
                       self.tag_name(), self.attributes(), self.namespaces())
            }
            NodeKind::PI(pi) => {
                write!(f, "PI {{ target: {:?}, value: {:?} }}", pi.target, pi.value)
            }
            NodeKind::Comment(text) => write!(f, "Comment({:?})", text),
            NodeKind::Text(ref text) => write!(f, "Text({:?})", text),
        }
    }
}

macro_rules! axis_iterators {
    ($(#[$m:meta] $i:ident($f:path);)*) => {
        $(
            #[$m]
            pub struct $i<'a, 'd: 'a>(Option<Node<'a, 'd>>);
            impl<'a, 'd: 'a> Clone for $i<'a, 'd> {
                fn clone(&self) -> Self {
                    $i(self.0)
                }
            }
            impl<'a, 'd: 'a> Iterator for $i<'a, 'd> {
                type Item = Node<'a, 'd>;
                fn next(&mut self) -> Option<Self::Item> {
                    let node = self.0.take();
                    self.0 = node.as_ref().and_then($f);
                    node
                }
            }
        )*
    };
}

axis_iterators! {
    /// Iterator over ancestors.
    Ancestors(Node::parent);

    /// Iterator over previous siblings.
    PrevSiblings(Node::prev_sibling);

    /// Iterator over next siblings.
    NextSiblings(Node::next_sibling);

    /// Iterator over first children.
    FirstChildren(Node::first_child);

    /// Iterator over last children.
    LastChildren(Node::last_child);
}


/// Iterator over children.
pub struct Children<'a, 'd: 'a> {
    front: Option<Node<'a, 'd>>,
    back: Option<Node<'a, 'd>>,
}

impl<'a, 'd: 'a> Clone for Children<'a, 'd> {
    fn clone(&self) -> Self {
        Self { front: self.front, back: self.back }
    }
}

impl<'a, 'd: 'a> Iterator for Children<'a, 'd> {
    type Item = Node<'a, 'd>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            let node = self.front.take();
            self.back = None;
            node
        } else {
            let node = self.front.take();
            self.front = node.as_ref().and_then(Node::next_sibling);
            node
        }
    }
}

impl<'a, 'd: 'a> DoubleEndedIterator for Children<'a, 'd> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back == self.front {
            let node = self.back.take();
            self.front = None;
            node
        } else {
            let node = self.back.take();
            self.back = node.as_ref().and_then(Node::prev_sibling);
            node
        }
    }
}


/// Open or close edge of a node.
#[derive(Debug)]
pub enum Edge<'a, 'd: 'a> {
    /// Open.
    Open(Node<'a, 'd>),
    /// Close.
    Close(Node<'a, 'd>),
}

impl<'a, 'd: 'a> Copy for Edge<'a, 'd> {}

impl<'a, 'd: 'a> Clone for Edge<'a, 'd> {
    fn clone(&self) -> Self { *self }
}

impl<'a, 'd: 'a> Eq for Edge<'a, 'd> {}

impl<'a, 'd: 'a> PartialEq for Edge<'a, 'd> {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Edge::Open(a), Edge::Open(b)) | (Edge::Close(a), Edge::Close(b)) => a == b,
            _ => false,
        }
    }
}


/// Iterator which traverses a subtree.
pub struct Traverse<'a, 'd: 'a> {
    root: Node<'a, 'd>,
    edge: Option<Edge<'a, 'd>>,
}

impl<'a, 'd: 'a> Clone for Traverse<'a, 'd> {
    fn clone(&self) -> Self {
        Self { root: self.root, edge: self.edge }
    }
}

impl<'a, 'd: 'a> Iterator for Traverse<'a, 'd> {
    type Item = Edge<'a, 'd>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.edge {
            Some(Edge::Open(node)) => {
                self.edge = Some(match node.first_child() {
                    Some(first_child) => Edge::Open(first_child),
                    None => Edge::Close(node),
                });
            }
            Some(Edge::Close(node)) => {
                if node == self.root {
                    self.edge = None;
                } else if let Some(next_sibling) = node.next_sibling() {
                    self.edge = Some(Edge::Open(next_sibling));
                } else {
                    self.edge = node.parent().map(Edge::Close);
                }
            }
            None => {
                self.edge = Some(Edge::Open(self.root));
            }
        }

        self.edge
    }
}


/// Iterator over a node and its descendants.
pub struct Descendants<'a, 'd: 'a>(Traverse<'a, 'd>);

impl<'a, 'd: 'a> Clone for Descendants<'a, 'd> {
    fn clone(&self) -> Self {
        Descendants(self.0.clone())
    }
}

impl<'a, 'd: 'a> Iterator for Descendants<'a, 'd> {
    type Item = Node<'a, 'd>;

    fn next(&mut self) -> Option<Self::Item> {
        for edge in &mut self.0 {
            if let Edge::Open(node) = edge {
                return Some(node);
            }
        }

        None
    }
}
