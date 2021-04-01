/*!
Represent an [XML 1.0](https://www.w3.org/TR/xml/) document as a read-only tree.

The root point of the documentations is [`Document::parse`].

You can find more details in the [README] and the [parsing doc].

The tree structure itself is a heavily modified <https://github.com/causal-agent/ego-tree>
License: ISC.

[`Document::parse`]: struct.Document.html#method.parse
[README]: https://github.com/RazrFalcon/roxmltree/blob/master/README.md
[parsing doc]: https://github.com/RazrFalcon/roxmltree/blob/master/docs/parsing.md
*/

#![doc(html_root_url = "https://docs.rs/roxmltree/0.14.1")]

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::num::NonZeroU32;
use core::ops::Deref;

pub use xmlparser::TextPos;

mod parse;
pub use crate::parse::*;


/// The <http://www.w3.org/XML/1998/namespace> URI.
pub const NS_XML_URI: &str = "http://www.w3.org/XML/1998/namespace";

/// The <http://www.w3.org/2000/xmlns/> URI.
pub const NS_XMLNS_URI: &str = "http://www.w3.org/2000/xmlns/";


type Range = core::ops::Range<usize>;

/// An XML tree container.
///
/// A tree consists of [`Nodes`].
/// There are no separate structs for each node type.
/// So you should check the current node type yourself via [`Node::node_type()`].
/// There are only [5 types](enum.NodeType.html):
/// Root, Element, PI, Comment and Text.
///
/// As you can see there are no XML declaration and CDATA types.
/// The XML declaration is basically skipped, since it doesn't contain any
/// valuable information (we support only UTF-8 anyway).
/// And CDATA will be converted into a Text node as is, without
/// any preprocessing (you can read more about it
/// [here](https://github.com/RazrFalcon/roxmltree/blob/master/docs/parsing.md)).
///
/// Also, the Text node data can be accessed from the text node itself or from
/// the parent element via [`Node::text()`] or [`Node::tail()`].
///
/// [`Nodes`]: struct.Node.html
/// [`Node::node_type()`]: struct.Node.html#method.node_type
/// [`Node::text()`]: struct.Node.html#method.text
/// [`Node::tail()`]: struct.Node.html#method.tail
pub struct Document<'input> {
    /// An original data.
    ///
    /// Required for `text_pos` methods.
    text: &'input str,
    nodes: Vec<NodeData<'input>>,
    attrs: Vec<Attribute<'input>>,
    namespaces: Namespaces<'input>,
}

impl<'input> Document<'input> {
    /// Returns the root node.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e/>").unwrap();
    /// assert!(doc.root().is_root());
    /// assert!(doc.root().first_child().unwrap().has_tag_name("e"));
    /// ```
    #[inline]
    pub fn root<'a>(&'a self) -> Node<'a, 'input> {
        Node { id: NodeId::new(0), d: &self.nodes[0], doc: self }
    }

    /// Returns the node of the tree with the given NodeId.
    ///
    /// Note: NodeId::new(0) represents the root node
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
    /// use roxmltree::NodeId;
    /// assert_eq!(doc.get_node(NodeId::new(0)).unwrap(), doc.root());
    /// assert_eq!(doc.get_node(NodeId::new(1)), doc.descendants().find(|n| n.has_tag_name("p")));
    /// assert_eq!(doc.get_node(NodeId::new(2)), doc.descendants().find(|n| n.is_text()));
    /// assert_eq!(doc.get_node(NodeId::new(3)), None);
    /// ```
    #[inline]
    pub fn get_node<'a>(&'a self, id: NodeId) -> Option<Node<'a, 'input>> {
        self.nodes.get(id.get_usize()).map(|data| Node { id, d: data, doc: self })
    }

    /// Returns the root element of the document.
    ///
    /// Unlike `root`, will return a first element node.
    ///
    /// The root element always exists.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<!-- comment --><e/>").unwrap();
    /// assert!(doc.root_element().has_tag_name("e"));
    /// ```
    #[inline]
    pub fn root_element<'a>(&'a self) -> Node<'a, 'input> {
        // `expect` is safe, because the `Document` is guarantee to have at least one element.
        self.root().first_element_child().expect("XML documents must contain a root element")
    }

    /// Returns an iterator over document's descendant nodes.
    ///
    /// Shorthand for `doc.root().descendants()`.
    #[inline]
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
    /// use roxmltree::*;
    ///
    /// let doc = Document::parse("\
    /// <!-- comment -->
    /// <e/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.text_pos_at(10), TextPos::new(1, 11));
    /// assert_eq!(doc.text_pos_at(9999), TextPos::new(2, 5));
    /// ```
    #[inline]
    pub fn text_pos_at(&self, pos: usize) -> TextPos {
        xmlparser::Stream::from(self.text).gen_text_pos_from(pos)
    }

    /// Returns the input text of the original document.
    ///
    /// # Examples
    ///
    /// ```
    /// use roxmltree::*;
    ///
    /// let doc = Document::parse("<e/>").unwrap();
    ///
    /// assert_eq!(doc.input_text(), "<e/>");
    /// ```
    #[inline]
    pub fn input_text(&self) -> &'input str {
        self.text
    }
}

impl<'input> fmt::Debug for Document<'input> {
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


/// A list of supported node types.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NodeType {
    /// The root node of the `Document`.
    Root,
    /// An element node.
    ///
    /// Only an element can have a tag name and attributes.
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
pub struct PI<'input> {
    pub target: &'input str,
    pub value: Option<&'input str>,
}


/// A short range.
///
/// Just like Range, but only for `u32` and copyable.
#[derive(Clone, Copy, Debug)]
struct ShortRange {
    start: u32,
    end: u32,
}

impl From<Range> for ShortRange {
    #[inline]
    fn from(range: Range) -> Self {
        // Casting to `u32` should be safe since we have a 4GiB input data limit.
        debug_assert!(range.start <= core::u32::MAX as usize);
        debug_assert!(range.end <= core::u32::MAX as usize);
        ShortRange::new(range.start as u32, range.end as u32)
    }
}

impl ShortRange {
    #[inline]
    fn new(start: u32, end: u32) -> Self {
        ShortRange { start, end }
    }

    #[inline]
    fn to_urange(self) -> Range {
        self.start as usize .. self.end as usize
    }
}


/// A node ID stored as `u32`.
///
/// An index into a `Tree`-internal `Vec`.
///
/// Note that this value should be used with care since `roxmltree` doesn't
/// check that `NodeId` actually belongs to a selected `Document`.
/// So you can end up in a situation, when `NodeId` produced by one `Document`
/// is used to select a node in another `Document`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct NodeId(NonZeroU32);

impl NodeId {
    /// Construct a new `NodeId` from a `u32`.
    ///
    /// `u32` is more than enough since we have a 4GiB input data limit anyway.
    #[inline]
    pub fn new(id: u32) -> Self {
        debug_assert!(id < core::u32::MAX);

        // We are using `NonZeroUsize` to reduce overhead of `Option<NodeId>`.
        NodeId(NonZeroU32::new(id + 1).unwrap())
    }

    /// Returns the `u32` representation of the `NodeId`.
    #[inline]
    pub fn get(self) -> u32 {
        self.0.get() as u32 - 1
    }

    /// Returns the `usize` representation of the `NodeId`.
    #[inline]
    pub fn get_usize(self) -> usize {
        self.get() as usize
    }
}

impl From<u32> for NodeId {
    #[inline]
    fn from(id: u32) -> Self {
        NodeId::new(id)
    }
}

impl From<usize> for NodeId {
    #[inline]
    fn from(id: usize) -> Self {
        // Casting to `u32` should be safe since we have a 4GiB input data limit.
        debug_assert!(id <= core::u32::MAX as usize);
        NodeId::new(id as u32)
    }
}


enum NodeKind<'input> {
    Root,
    Element {
        tag_name: ExpandedNameOwned<'input>,
        attributes: ShortRange,
        namespaces: ShortRange,
    },
    PI(PI<'input>),
    Comment(&'input str),
    Text(Cow<'input, str>),
}


struct NodeData<'input> {
    parent: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    next_subtree: Option<NodeId>,
    last_child: Option<NodeId>,
    kind: NodeKind<'input>,
    range: ShortRange,
}


/// An attribute.
#[derive(Clone)]
pub struct Attribute<'input> {
    name: ExpandedNameOwned<'input>,
    value: Cow<'input, str>,
    range: ShortRange,
    value_range: ShortRange,
}

impl<'input> Attribute<'input> {
    /// Returns attribute's namespace URI.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org' a='b' n:a='c'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().attributes()[0].namespace(), None);
    /// assert_eq!(doc.root_element().attributes()[1].namespace(), Some("http://www.w3.org"));
    /// ```
    #[inline]
    pub fn namespace(&self) -> Option<&str> {
        self.name.ns.as_ref().map(Cow::as_ref)
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
    #[inline]
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
    #[inline]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns attribute's name range in bytes in the original document.
    ///
    /// You can calculate a human-readable text position via [Document::text_pos_at].
    ///
    /// ```text
    /// <e attr='value'/>
    ///    ^
    /// ```
    ///
    /// [Document::text_pos_at]: struct.Document.html#method.text_pos_at
    #[inline]
    pub fn range(&self) -> Range {
        self.range.to_urange()
    }

    /// Returns attribute's value range in bytes in the original document.
    ///
    /// You can calculate a human-readable text position via [Document::text_pos_at].
    ///
    /// ```text
    /// <e attr='value'/>
    ///          ^
    /// ```
    ///
    /// [Document::text_pos_at]: struct.Document.html#method.text_pos_at
    #[inline]
    pub fn value_range(&self) -> Range {
        self.value_range.to_urange()
    }
}

impl<'input> PartialEq for Attribute<'input> {
    #[inline]
    fn eq(&self, other: &Attribute<'input>) -> bool {
        self.name == other.name && self.value == other.value
    }
}

impl<'input> fmt::Debug for Attribute<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Attribute {{ name: {:?}, value: {:?} }}",
               self.name, self.value)
    }
}


/// A namespace.
///
/// Contains URI and *prefix* pair.
#[derive(Clone, PartialEq, Debug)]
pub struct Namespace<'input> {
    name: Option<&'input str>,
    uri: Cow<'input, str>,
}

impl<'input> Namespace<'input> {
    /// Returns namespace name/prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns:n='http://www.w3.org'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().namespaces()[0].name(), Some("n"));
    /// ```
    ///
    /// ```
    /// let doc = roxmltree::Document::parse(
    ///     "<e xmlns='http://www.w3.org'/>"
    /// ).unwrap();
    ///
    /// assert_eq!(doc.root_element().namespaces()[0].name(), None);
    /// ```
    #[inline]
    pub fn name(&self) -> Option<&str> {
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
    #[inline]
    pub fn uri(&self) -> &str {
        self.uri.as_ref()
    }
}


struct Namespaces<'input>(Vec<Namespace<'input>>);

impl<'input> Namespaces<'input> {
    #[inline]
    fn push_ns(&mut self, name: Option<&'input str>, uri: Cow<'input, str>) {
        debug_assert_ne!(name, Some(""));
        self.0.push(Namespace { name, uri });
    }

    #[inline]
    fn exists(&self, start: usize, prefix: Option<&str>) -> bool {
        self[start..].iter().any(|ns| ns.name == prefix)
    }
}

impl<'input> Deref for Namespaces<'input> {
    type Target = Vec<Namespace<'input>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[derive(Clone, PartialEq)]
struct ExpandedNameOwned<'input> {
    ns: Option<Cow<'input, str>>,
    prefix: &'input str, // Used only for closing tags matching during parsing.
    name: &'input str,
}

impl<'a, 'input> ExpandedNameOwned<'input> {
    #[inline]
    fn as_ref(&'a self) -> ExpandedName<'a, 'input> {
        ExpandedName {
            uri: self.ns.as_ref().map(Cow::as_ref),
            name: self.name,
        }
    }
}

impl<'input> fmt::Debug for ExpandedNameOwned<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.ns {
            Some(ref ns) => write!(f, "{{{}}}{}", ns.as_ref(), self.name),
            None => write!(f, "{}", self.name),
        }
    }
}


/// An expanded name.
///
/// Contains an namespace URI and name pair.
#[derive(Clone, Copy, PartialEq)]
pub struct ExpandedName<'a, 'b> {
    uri: Option<&'a str>,
    name: &'b str,
}

impl<'a, 'b> ExpandedName<'a, 'b> {
    /// Returns a namespace URI.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().tag_name().namespace(), Some("http://www.w3.org"));
    /// ```
    #[inline]
    pub fn namespace(&self) -> Option<&'a str> {
        self.uri
    }

    /// Returns a local name.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().tag_name().name(), "e");
    /// ```
    #[inline]
    pub fn name(&self) -> &'b str {
        self.name
    }
}

impl fmt::Debug for ExpandedName<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.namespace() {
            Some(ns) => write!(f, "{{{}}}{}", ns, self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

impl<'a, 'b> From<&'b str> for ExpandedName<'a, 'b> {
    #[inline]
    fn from(v: &'b str) -> Self {
        ExpandedName {
            uri: None,
            name: v,
        }
    }
}

impl<'a, 'b> From<(&'a str, &'b str)> for ExpandedName<'a, 'b> {
    #[inline]
    fn from(v: (&'a str, &'b str)) -> Self {
        ExpandedName {
            uri: Some(v.0),
            name: v.1,
        }
    }
}


/// A node in a document.
///
/// # Document Order
///
/// The implementation of the `Ord` traits for `Node` is based on the concept of *document-order*.
/// In layman's terms, document-order is the order in which one would see each element if
/// one opened a document in a text editor or web browser and scrolled down.
/// Document-order convention is followed in XPath, CSS Counters, and DOM selectors API
/// to ensure consistent results from selection.
/// One difference in `roxmltree` is that there is the notion of more than one document
/// in existence at a time. While Nodes within the same document are in document-order,
/// Nodes in different documents will be grouped together, but not in any particular
/// order.
///
/// As an example, if we have a Document `a` with Nodes `[a0, a1, a2]` and a
/// Document `b` with Nodes `[b0, b1]`, these Nodes in order could be either
/// `[a0, a1, a2, b0, b1]` or `[b0, b1, a0, a1, a2]` and roxmltree makes no
/// guarantee which it will be.
///
/// Document-order is defined here in the
/// [W3C XPath Recommendation](https://www.w3.org/TR/xpath-3/#id-document-order)
/// The use of document-order in DOM Selectors is described here in the
/// [W3C Selectors API Level 1](https://www.w3.org/TR/selectors-api/#the-apis)
#[derive(Clone, Copy)]
pub struct Node<'a, 'input: 'a> {
    /// Node's ID.
    id: NodeId,

    /// The tree containing the node.
    doc: &'a Document<'input>,

    /// Node's data.
    d: &'a NodeData<'input>,
}

impl Eq for Node<'_, '_> {}

impl PartialEq for Node<'_, '_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
           self.id == other.id
        && self.doc as *const _ == other.doc as *const _
        && self.d as *const _ == other.d as *const _
    }
}

impl PartialOrd for Node<'_, '_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node<'_, '_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let id_cmp = self.id.0.cmp(&other.id.0);
        match id_cmp {
            Ordering::Equal => {
                let this_doc_ptr = self.doc as *const Document;
                let other_doc_ptr = other.doc as *const Document;
                this_doc_ptr.cmp(&other_doc_ptr)
            }
            _ => id_cmp
        }
    }
}

impl Hash for Node<'_, '_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.0.hash(state);
        (self.doc as *const Document).hash(state);
        (self.d as *const NodeData).hash(state);
    }
}

impl<'a, 'input: 'a> Node<'a, 'input> {
    /// Returns node's type.
    #[inline]
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
    #[inline]
    pub fn is_root(&self) -> bool {
        self.node_type() == NodeType::Root
    }

    /// Checks that node is an element node.
    #[inline]
    pub fn is_element(&self) -> bool {
        self.node_type() == NodeType::Element
    }

    /// Checks that node is a processing instruction node.
    #[inline]
    pub fn is_pi(&self) -> bool {
        self.node_type() == NodeType::PI
    }

    /// Checks that node is a comment node.
    #[inline]
    pub fn is_comment(&self) -> bool {
        self.node_type() == NodeType::Comment
    }

    /// Checks that node is a text node.
    #[inline]
    pub fn is_text(&self) -> bool {
        self.node_type() == NodeType::Text
    }

    /// Returns node's document.
    #[inline]
    pub fn document(&self) -> &'a Document<'input> {
        self.doc
    }

    /// Returns node's tag name.
    ///
    /// Returns an empty name with no namespace if the current node is not an element.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().tag_name().namespace(), Some("http://www.w3.org"));
    /// assert_eq!(doc.root_element().tag_name().name(), "e");
    /// ```
    #[inline]
    pub fn tag_name(&self) -> ExpandedName<'a, 'input> {
        match self.d.kind {
            NodeKind::Element { ref tag_name, .. } => tag_name.as_ref(),
            _ => "".into()
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
    pub fn has_tag_name<'n, 'm, N>(&self, name: N) -> bool
    where
        N: Into<ExpandedName<'n, 'm>>,
    {
        let name = name.into();

        match self.d.kind {
            NodeKind::Element { ref tag_name, .. } => {
                match name.namespace() {
                    Some(_) => tag_name.as_ref() == name,
                    None => tag_name.name == name.name,
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
    pub fn default_namespace(&self) -> Option<&'a str> {
        self.namespaces().iter().find(|ns| ns.name.is_none()).map(|v| v.uri.as_ref())
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
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns:n=''/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().lookup_prefix(""), Some("n"));
    /// ```
    pub fn lookup_prefix(&self, uri: &str) -> Option<&'a str> {
        if uri == NS_XML_URI {
            return Some("xml");
        }

        self.namespaces().iter().find(|ns| ns.uri == uri).map(|v| v.name).unwrap_or(None)
    }

    /// Returns an URI for a given prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns:n='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().lookup_namespace_uri(Some("n")), Some("http://www.w3.org"));
    /// ```
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e xmlns='http://www.w3.org'/>").unwrap();
    ///
    /// assert_eq!(doc.root_element().lookup_namespace_uri(None), Some("http://www.w3.org"));
    /// ```
    pub fn lookup_namespace_uri(&self, prefix: Option<&'a str>) -> Option<&'a str> {
        self.namespaces().iter().find(|ns| ns.name == prefix).map(|v| v.uri.as_ref())
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
    pub fn attribute<'n, 'm, N>(&self, name: N) -> Option<&'a str>
    where
        N: Into<ExpandedName<'n, 'm>>,
    {
        let name = name.into();
        self.attributes().iter().find(|a| a.name.as_ref() == name).map(|a| a.value.as_ref())
    }

    /// Returns element's attribute object.
    ///
    /// The same as [`attribute()`], but returns the `Attribute` itself instead of a value string.
    ///
    /// [`attribute()`]: struct.Node.html#method.attribute
    pub fn attribute_node<'n, 'm, N>(&self, name: N) -> Option<&'a Attribute<'input>>
    where
        N: Into<ExpandedName<'n, 'm>>,
    {
        let name = name.into();
        self.attributes().iter().find(|a| a.name.as_ref() == name)
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
    pub fn has_attribute<'n, 'm, N>(&self, name: N) -> bool
    where
        N: Into<ExpandedName<'n, 'm>>,
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
    #[inline]
    pub fn attributes(&self) -> &'a [Attribute<'input>] {
        match self.d.kind {
            NodeKind::Element { ref attributes, .. } => &self.doc.attrs[attributes.to_urange()],
            _ => &[],
        }
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
    #[inline]
    pub fn namespaces(&self) -> &'a [Namespace<'input>] {
        match self.d.kind {
            NodeKind::Element { ref namespaces, .. } => {
                &self.doc.namespaces[namespaces.to_urange()]
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
    #[inline]
    pub fn text(&self) -> Option<&'a str> {
        match self.d.kind {
            NodeKind::Element { .. } => {
                match self.first_child() {
                    Some(child) if child.is_text() => {
                        match self.doc.nodes[child.id.get_usize()].kind {
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
    #[inline]
    pub fn tail(&self) -> Option<&'a str> {
        if !self.is_element() {
            return None;
        }

        match self.next_sibling().map(|n| n.id) {
            Some(id) => {
                match self.doc.nodes[id.get_usize()].kind {
                    NodeKind::Text(ref text) => Some(text),
                    _ => None
                }
            }
            None => None,
        }
    }

    /// Returns node as Processing Instruction.
    #[inline]
    pub fn pi(&self) -> Option<PI<'input>> {
        match self.d.kind {
            NodeKind::PI(pi) => Some(pi),
            _ => None,
        }
    }

    /// Returns the parent of this node.
    #[inline]
    pub fn parent(&self) -> Option<Self> {
        self.d.parent.map(|id| self.doc.get_node(id).unwrap())
    }

    /// Returns the parent element of this node.
    pub fn parent_element(&self) -> Option<Self> {
        self.ancestors().skip(1).find(|n| n.is_element())
    }

    /// Returns the previous sibling of this node.
    #[inline]
    pub fn prev_sibling(&self) -> Option<Self> {
        self.d.prev_sibling.map(|id| self.doc.get_node(id).unwrap())
    }

    /// Returns the previous sibling element of this node.
    pub fn prev_sibling_element(&self) -> Option<Self> {
        self.prev_siblings().skip(1).find(|n| n.is_element())
    }

    /// Returns the next sibling of this node.
    #[inline]
    pub fn next_sibling(&self) -> Option<Self> {
        self.d.next_subtree
            .map(|id| self.doc.get_node(id).unwrap())
            .and_then(|node| {
                let possibly_self = node.d.prev_sibling
                    .expect("next_subtree will always have a previous sibling");
                if possibly_self == self.id { Some(node) } else { None }
            })
    }

    /// Returns the next sibling element of this node.
    pub fn next_sibling_element(&self) -> Option<Self> {
        self.next_siblings().skip(1).find(|n| n.is_element())
    }

    /// Returns the first child of this node.
    #[inline]
    pub fn first_child(&self) -> Option<Self> {
        self.d.last_child.map(|_| self.doc.get_node(NodeId::new(self.id.get() + 1)).unwrap())
    }

    /// Returns the first element child of this node.
    pub fn first_element_child(&self) -> Option<Self> {
        self.children().find(|n| n.is_element())
    }

    /// Returns the last child of this node.
    #[inline]
    pub fn last_child(&self) -> Option<Self> {
        self.d.last_child.map(|id| self.doc.get_node(id).unwrap())
    }

    /// Returns the last element child of this node.
    pub fn last_element_child(&self) -> Option<Self> {
        self.children().filter(|n| n.is_element()).last()
    }

    /// Returns true if this node has siblings.
    #[inline]
    pub fn has_siblings(&self) -> bool {
        self.d.prev_sibling.is_some() || self.next_sibling().is_some()
    }

    /// Returns true if this node has children.
    #[inline]
    pub fn has_children(&self) -> bool {
        self.d.last_child.is_some()
    }

    /// Returns an iterator over ancestor nodes starting at this node.
    #[inline]
    pub fn ancestors(&self) -> AxisIter<'a, 'input> {
        AxisIter { node: Some(*self), next: Node::parent }
    }

    /// Returns an iterator over previous sibling nodes starting at this node.
    #[inline]
    pub fn prev_siblings(&self) -> AxisIter<'a, 'input> {
        AxisIter { node: Some(*self), next: Node::prev_sibling }
    }

    /// Returns an iterator over next sibling nodes starting at this node.
    #[inline]
    pub fn next_siblings(&self) -> AxisIter<'a, 'input> {
        AxisIter { node: Some(*self), next: Node::next_sibling }
    }

    /// Returns an iterator over first children nodes starting at this node.
    #[inline]
    pub fn first_children(&self) -> AxisIter<'a, 'input> {
        AxisIter { node: Some(*self), next: Node::first_child }
    }

    /// Returns an iterator over last children nodes starting at this node.
    #[inline]
    pub fn last_children(&self) -> AxisIter<'a, 'input> {
        AxisIter { node: Some(*self), next: Node::last_child }
    }

    /// Returns an iterator over children nodes.
    #[inline]
    pub fn children(&self) -> Children<'a, 'input> {
        Children { front: self.first_child(), back: self.last_child() }
    }

    /// Returns an iterator over this node and its descendants.
    #[inline]
    pub fn descendants(&self) -> Descendants<'a, 'input> {
        Descendants::new(*self)
    }

    /// Returns node's range in bytes in the original document.
    #[inline]
    pub fn range(&self) -> Range {
        self.d.range.to_urange()
    }

    /// Returns node's NodeId
    #[inline]
    pub fn id(&self) -> NodeId {
        self.id
    }
}

impl<'a, 'input: 'a> fmt::Debug for Node<'a, 'input> {
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


/// Iterator over specified axis.
#[derive(Clone)]
pub struct AxisIter<'a, 'input: 'a> {
    node: Option<Node<'a, 'input>>,
    next: fn(&Node<'a, 'input>) -> Option<Node<'a, 'input>>,
}

impl<'a, 'input: 'a> Iterator for AxisIter<'a, 'input> {
    type Item = Node<'a, 'input>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node.take();
        self.node = node.as_ref().and_then(self.next);
        node
    }
}

impl fmt::Debug for AxisIter<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Pixmap")
            .field("node", &self.node)
            .field("next", &"fn()")
            .finish()
    }
}


/// Iterator over children.
#[derive(Clone, Debug)]
pub struct Children<'a, 'input: 'a> {
    front: Option<Node<'a, 'input>>,
    back: Option<Node<'a, 'input>>,
}

impl<'a, 'input: 'a> Iterator for Children<'a, 'input> {
    type Item = Node<'a, 'input>;

    #[inline]
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

impl<'a, 'input: 'a> DoubleEndedIterator for Children<'a, 'input> {
    #[inline]
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


/// Iterator over a node and its descendants.
#[derive(Clone, Debug)]
pub struct Descendants<'a, 'input> {
    doc: &'a Document<'input>,
    current: NodeId,
    until: NodeId,
}

impl<'a, 'input> Descendants<'a, 'input> {
    #[inline]
    fn new(start: Node<'a, 'input>) -> Self {
        Self {
            doc: &start.doc,
            current: start.id,
            until: start.d.next_subtree.unwrap_or_else(|| NodeId::from(start.doc.nodes.len()))
        }
    }
}

impl<'a, 'input> Iterator for Descendants<'a, 'input> {
    type Item = Node<'a, 'input>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = if self.current == self.until {
            None
        } else {
            Some(self.doc.get_node(self.current).unwrap())
        };

        self.current = NodeId::new(self.current.get() + 1);
        next
    }
}
