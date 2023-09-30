use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::Range;

use xmlparser::{self, Reference, StrSpan, Stream, TextPos};

use crate::{
    AttributeData, Document, ExpandedNameIndexed, NamespaceIdx, Namespaces, NodeData, NodeId,
    NodeKind, ShortRange, StringStorage, NS_XMLNS_URI, NS_XML_PREFIX, NS_XML_URI, PI, XMLNS,
};

/// A list of all possible errors.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// The `xmlns:xml` attribute must have an <http://www.w3.org/XML/1998/namespace> URI.
    InvalidXmlPrefixUri(TextPos),

    /// Only the `xmlns:xml` attribute can have the <http://www.w3.org/XML/1998/namespace> URI.
    UnexpectedXmlUri(TextPos),

    /// The <http://www.w3.org/2000/xmlns/> URI must not be declared.
    UnexpectedXmlnsUri(TextPos),

    /// `xmlns` can't be used as an element prefix.
    InvalidElementNamePrefix(TextPos),

    /// A namespace was already defined on this element.
    DuplicatedNamespace(String, TextPos),

    /// An unknown namespace.
    ///
    /// Indicates that an element or an attribute has an unknown qualified name prefix.
    ///
    /// The first value is a prefix.
    UnknownNamespace(String, TextPos),

    /// Incorrect tree structure.
    #[allow(missing_docs)]
    UnexpectedCloseTag {
        expected: String,
        actual: String,
        pos: TextPos,
    },

    /// Entity value starts with a close tag.
    ///
    /// Example:
    /// ```xml
    /// <!DOCTYPE test [ <!ENTITY p '</p>'> ]>
    /// <root>&p;</root>
    /// ```
    UnexpectedEntityCloseTag(TextPos),

    /// A reference to an entity that was not defined in the DTD.
    UnknownEntityReference(String, TextPos),

    /// A malformed entity reference.
    ///
    /// A `&` character inside an attribute value or text indicates an entity reference.
    /// Otherwise, the document is not well-formed.
    MalformedEntityReference(TextPos),

    /// A possible entity reference loop.
    ///
    /// The current depth limit is 10. The max number of references per reference is 255.
    EntityReferenceLoop(TextPos),

    /// Attribute value cannot have a `<` character.
    InvalidAttributeValue(TextPos),

    /// An element has a duplicated attributes.
    ///
    /// This also includes namespaces resolving.
    /// So an element like this will lead to an error.
    /// ```xml
    /// <e xmlns:n1='http://www.w3.org' xmlns:n2='http://www.w3.org' n1:a='b1' n2:a='b2'/>
    /// ```
    DuplicatedAttribute(String, TextPos),

    /// The XML document must have at least one element.
    NoRootNode,

    /// The root node was opened but never closed.
    UnclosedRootNode,

    /// An XML with DTD detected.
    ///
    /// This error will be emitted only when `ParsingOptions::allow_dtd` is set to `false`.
    DtdDetected,

    /// Indicates that the [`ParsingOptions::nodes_limit`] was reached.
    NodesLimitReached,

    /// Indicates that too many attributes were parsed.
    AttributesLimitReached,

    /// Indicates that too many namespaces were parsed.
    NamespacesLimitReached,

    /// Errors detected by the `xmlparser` crate.
    ParserError(xmlparser::Error),
}

impl Error {
    /// Returns the error position.
    pub fn pos(&self) -> TextPos {
        match *self {
            Error::InvalidXmlPrefixUri(pos) => pos,
            Error::UnexpectedXmlUri(pos) => pos,
            Error::UnexpectedXmlnsUri(pos) => pos,
            Error::InvalidElementNamePrefix(pos) => pos,
            Error::DuplicatedNamespace(ref _name, pos) => pos,
            Error::UnknownNamespace(ref _name, pos) => pos,
            Error::UnexpectedCloseTag { pos, .. } => pos,
            Error::UnexpectedEntityCloseTag(pos) => pos,
            Error::UnknownEntityReference(ref _name, pos) => pos,
            Error::MalformedEntityReference(pos) => pos,
            Error::EntityReferenceLoop(pos) => pos,
            Error::InvalidAttributeValue(pos) => pos,
            Error::DuplicatedAttribute(ref _name, pos) => pos,
            Error::ParserError(ref err) => err.pos(),
            _ => TextPos::new(1, 1),
        }
    }
}

impl From<xmlparser::Error> for Error {
    fn from(e: xmlparser::Error) -> Self {
        Error::ParserError(e)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Error::InvalidXmlPrefixUri(pos) => {
                write!(f, "'xml' namespace prefix mapped to wrong URI at {}", pos)
            }
            Error::UnexpectedXmlUri(pos) => {
                write!(
                    f,
                    "the 'xml' namespace URI is used for not 'xml' prefix at {}",
                    pos
                )
            }
            Error::UnexpectedXmlnsUri(pos) => {
                write!(
                    f,
                    "the 'xmlns' URI is used at {}, but it must not be declared",
                    pos
                )
            }
            Error::InvalidElementNamePrefix(pos) => {
                write!(
                    f,
                    "the 'xmlns' prefix is used at {}, but it must not be",
                    pos
                )
            }
            Error::DuplicatedNamespace(ref name, pos) => {
                write!(f, "namespace '{}' at {} is already defined", name, pos)
            }
            Error::UnknownNamespace(ref name, pos) => {
                write!(f, "an unknown namespace prefix '{}' at {}", name, pos)
            }
            Error::UnexpectedCloseTag {
                ref expected,
                ref actual,
                pos,
            } => {
                write!(
                    f,
                    "expected '{}' tag, not '{}' at {}",
                    expected, actual, pos
                )
            }
            Error::UnexpectedEntityCloseTag(pos) => {
                write!(f, "unexpected close tag at {}", pos)
            }
            Error::MalformedEntityReference(pos) => {
                write!(f, "malformed entity reference at {}", pos)
            }
            Error::UnknownEntityReference(ref name, pos) => {
                write!(f, "unknown entity reference '{}' at {}", name, pos)
            }
            Error::EntityReferenceLoop(pos) => {
                write!(f, "a possible entity reference loop is detected at {}", pos)
            }
            Error::InvalidAttributeValue(pos) => {
                write!(f, "unescaped '<' found at {}", pos)
            }
            Error::DuplicatedAttribute(ref name, pos) => {
                write!(f, "attribute '{}' at {} is already defined", name, pos)
            }
            Error::NoRootNode => {
                write!(f, "the document does not have a root node")
            }
            Error::UnclosedRootNode => {
                write!(f, "the root node was opened but never closed")
            }
            Error::DtdDetected => {
                write!(f, "XML with DTD detected")
            }
            Error::NodesLimitReached => {
                write!(f, "nodes limit reached")
            }
            Error::AttributesLimitReached => {
                write!(f, "more than 2^32 attributes were parsed")
            }
            Error::NamespacesLimitReached => {
                write!(f, "more than 2^16 unique namespaces were parsed")
            }
            Error::ParserError(ref err) => {
                write!(f, "{}", err)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn description(&self) -> &str {
        "an XML parsing error"
    }
}

/// Parsing options.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParsingOptions {
    /// Allow DTD parsing.
    ///
    /// When set to `false`, XML with DTD will cause an error.
    /// Empty DTD block is not an error.
    ///
    /// Currently, there is no option to simply skip DTD.
    /// Mainly because you will get `UnknownEntityReference` error later anyway.
    ///
    /// This flag is set to `false` by default for security reasons,
    /// but `roxmltree` still has checks for billion laughs attack,
    /// so this is just an extra security measure.
    ///
    /// Default: false
    pub allow_dtd: bool,

    /// Sets the maximum number of nodes to parse.
    ///
    /// Useful when dealing with random input to limit memory usage.
    ///
    /// Default: u32::MAX (no limit)
    pub nodes_limit: u32,
}

// Explicit for readability.
#[allow(clippy::derivable_impls)]
impl Default for ParsingOptions {
    fn default() -> Self {
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: core::u32::MAX,
        }
    }
}

struct TempAttributeData<'input> {
    prefix: StrSpan<'input>,
    local: StrSpan<'input>,
    value: StringStorage<'input>,
    #[cfg(feature = "positions")]
    pos: usize,
}

impl<'input> Document<'input> {
    /// Parses the input XML string.
    ///
    /// We do not support `&[u8]` or `Reader` because the input must be an already allocated
    /// UTF-8 string.
    ///
    /// This is a shorthand for `Document::parse_with_options(data, ParsingOptions::default())`.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e/>").unwrap();
    /// assert_eq!(doc.descendants().count(), 2); // root node + `e` element node
    /// ```
    #[inline]
    pub fn parse(text: &str) -> Result<Document, Error> {
        Self::parse_with_options(text, ParsingOptions::default())
    }

    /// Parses the input XML string using to selected options.
    ///
    /// We do not support `&[u8]` or `Reader` because the input must be an already allocated
    /// UTF-8 string.
    ///
    /// # Examples
    ///
    /// ```
    /// let opt = roxmltree::ParsingOptions::default();
    /// let doc = roxmltree::Document::parse_with_options("<e/>", opt).unwrap();
    /// assert_eq!(doc.descendants().count(), 2); // root node + `e` element node
    /// ```
    #[inline]
    pub fn parse_with_options(text: &str, opt: ParsingOptions) -> Result<Document, Error> {
        parse(text, opt)
    }

    fn append(
        &mut self,
        kind: NodeKind<'input>,
        range: Range<usize>,
        state: &mut ParserState<'input>,
    ) -> Result<NodeId, Error> {
        if self.nodes.len() >= state.opt.nodes_limit as usize {
            return Err(Error::NodesLimitReached);
        }

        #[cfg(not(feature = "positions"))]
        let _ = range;

        let new_child_id = NodeId::from(self.nodes.len());

        let appending_element = match kind {
            NodeKind::Element { .. } => true,
            _ => false,
        };

        self.nodes.push(NodeData {
            parent: Some(state.parent_id),
            prev_sibling: None,
            next_subtree: None,
            last_child: None,
            kind,
            #[cfg(feature = "positions")]
            range,
        });

        let last_child_id = self.nodes[state.parent_id.get_usize()].last_child;
        self.nodes[new_child_id.get_usize()].prev_sibling = last_child_id;
        self.nodes[state.parent_id.get_usize()].last_child = Some(new_child_id);

        state.awaiting_subtree.iter().for_each(|id| {
            self.nodes[id.get_usize()].next_subtree = Some(new_child_id);
        });
        state.awaiting_subtree.clear();

        if !appending_element {
            state
                .awaiting_subtree
                .push(NodeId::from(self.nodes.len() - 1));
        }

        Ok(new_child_id)
    }
}

struct Entity<'input> {
    name: &'input str,
    value: StrSpan<'input>,
}

struct ParserState<'input> {
    opt: ParsingOptions,
    namespace_start_idx: usize,
    current_attributes: Vec<TempAttributeData<'input>>,
    awaiting_subtree: Vec<NodeId>,
    parent_prefixes: Vec<&'input str>,
    entities: Vec<Entity<'input>>,
    after_text: bool,
    parent_id: NodeId,
}

#[derive(Clone, Copy)]
struct TagNameSpan<'input> {
    prefix: StrSpan<'input>,
    name: StrSpan<'input>,
    span: StrSpan<'input>,
}

impl<'input> TagNameSpan<'input> {
    #[inline]
    fn new_null() -> Self {
        Self {
            prefix: StrSpan::from(""),
            name: StrSpan::from(""),
            span: StrSpan::from(""),
        }
    }

    #[inline]
    fn new(prefix: StrSpan<'input>, name: StrSpan<'input>, span: StrSpan<'input>) -> Self {
        Self { prefix, name, span }
    }
}

/// An entity loop detector.
///
/// Limits:
/// - Entities depth is 10.
/// - Maximum number of entity references per entity reference is 255.
///
/// Basically, if a text or an attribute has an entity reference and this reference
/// has more than 10 nested references - this is an error.
///
/// This is useful for simple loops like:
///
/// ```text
/// <!ENTITY a '&b;'>
/// <!ENTITY b '&a;'>
/// ```
///
/// And, if a text or an attribute has an entity reference and it references more
/// than 255 references - this is an error.
///
/// This is useful for cases like billion laughs attack, where depth can be pretty small,
/// but the number of references is exponentially increasing:
///
/// ```text
/// <!ENTITY lol "lol">
/// <!ENTITY lol1 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
/// <!ENTITY lol2 "&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;">
/// <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
/// <!ENTITY lol4 "&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;">
/// ```
#[derive(Default)]
struct LoopDetector {
    /// References depth.
    depth: u8,
    /// Number of references resolved by the root reference.
    references: u8,
}

impl LoopDetector {
    #[inline]
    fn inc_depth(&mut self, stream: &Stream) -> Result<(), Error> {
        if self.depth < 10 {
            self.depth += 1;
            Ok(())
        } else {
            Err(Error::EntityReferenceLoop(stream.gen_text_pos()))
        }
    }

    #[inline]
    fn dec_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }

        // Reset references count after reaching zero depth.
        if self.depth == 0 {
            self.references = 0;
        }
    }

    #[inline]
    fn inc_references(&mut self, stream: &Stream) -> Result<(), Error> {
        if self.depth == 0 {
            // Allow infinite amount of references at zero depth.
            Ok(())
        } else {
            if self.references == core::u8::MAX {
                return Err(Error::EntityReferenceLoop(stream.gen_text_pos()));
            }

            self.references += 1;
            Ok(())
        }
    }
}

fn parse(text: &str, opt: ParsingOptions) -> Result<Document, Error> {
    let mut state = ParserState {
        opt,
        namespace_start_idx: 1,
        current_attributes: Vec::with_capacity(16),
        entities: Vec::new(),
        awaiting_subtree: Vec::new(),
        parent_prefixes: Vec::new(),
        after_text: false,
        parent_id: NodeId::new(0),
    };
    let mut text_buffer = TextBuffer::new();

    // Trying to guess rough nodes and attributes amount.
    let nodes_capacity = text.bytes().filter(|c| *c == b'<').count();
    let attributes_capacity = text.bytes().filter(|c| *c == b'=').count();

    // Init document.
    let mut doc = Document {
        text,
        nodes: Vec::with_capacity(nodes_capacity),
        attributes: Vec::with_capacity(attributes_capacity),
        namespaces: Namespaces::default(),
    };

    // Add a root node.
    doc.nodes.push(NodeData {
        parent: None,
        prev_sibling: None,
        next_subtree: None,
        last_child: None,
        kind: NodeKind::Root,
        #[cfg(feature = "positions")]
        range: 0..text.len(),
    });

    doc.namespaces
        .push_ns(Some(NS_XML_PREFIX), BorrowedText::Input(NS_XML_URI))?;

    let parser = xmlparser::Tokenizer::from(text);
    state.parent_prefixes.push("");
    let mut tag_name = TagNameSpan::new_null();
    process_tokens(
        parser,
        &mut LoopDetector::default(),
        &mut tag_name,
        &mut text_buffer,
        &mut state,
        &mut doc,
    )?;

    if !doc.root().children().any(|n| n.is_element()) {
        return Err(Error::NoRootNode);
    }

    if state.parent_prefixes.len() > 1 {
        return Err(Error::UnclosedRootNode);
    }

    doc.nodes.shrink_to_fit();
    doc.attributes.shrink_to_fit();
    doc.namespaces.shrink_to_fit();

    Ok(doc)
}

#[allow(clippy::collapsible_match)]
fn process_tokens<'input>(
    parser: xmlparser::Tokenizer<'input>,
    loop_detector: &mut LoopDetector,
    tag_name: &mut TagNameSpan<'input>,
    text_buffer: &mut TextBuffer,
    state: &mut ParserState<'input>,
    doc: &mut Document<'input>,
) -> Result<(), Error> {
    for token in parser {
        let token = token?;
        match token {
            xmlparser::Token::ProcessingInstruction {
                target,
                content,
                span,
            } => {
                let pi = NodeKind::PI(PI {
                    target: target.as_str(),
                    value: content.map(|v| v.as_str()),
                });
                doc.append(pi, span.range(), state)?;
            }
            xmlparser::Token::Comment { text, span } => {
                doc.append(
                    NodeKind::Comment(StringStorage::Borrowed(text.as_str())),
                    span.range(),
                    state,
                )?;
            }
            xmlparser::Token::Text { text } => {
                process_text(text, loop_detector, text_buffer, state, doc)?;
            }
            xmlparser::Token::Cdata { text, span } => {
                process_cdata(text, span, text_buffer, state, doc)?;
            }
            xmlparser::Token::ElementStart {
                prefix,
                local,
                span,
            } => {
                if prefix.as_str() == XMLNS {
                    let pos = err_pos_from_span(doc.text, prefix);
                    return Err(Error::InvalidElementNamePrefix(pos));
                }

                *tag_name = TagNameSpan::new(prefix, local, span);
            }
            xmlparser::Token::Attribute {
                prefix,
                local,
                value,
                span,
            } => {
                process_attribute(
                    prefix,
                    local,
                    value,
                    span,
                    loop_detector,
                    text_buffer,
                    state,
                    doc,
                )?;
            }
            xmlparser::Token::ElementEnd { end, span } => {
                process_element(*tag_name, end, span, state, doc)?;
            }
            xmlparser::Token::DtdStart { .. } => {
                if !state.opt.allow_dtd {
                    return Err(Error::DtdDetected);
                }
            }
            xmlparser::Token::EntityDeclaration {
                name, definition, ..
            } => {
                if let xmlparser::EntityDefinition::EntityValue(value) = definition {
                    state.entities.push(Entity {
                        name: name.as_str(),
                        value,
                    });
                }
            }
            _ => {}
        }

        match token {
            xmlparser::Token::ProcessingInstruction { .. }
            | xmlparser::Token::Comment { .. }
            | xmlparser::Token::ElementStart { .. }
            | xmlparser::Token::ElementEnd { .. } => {
                state.after_text = false;
            }
            _ => {}
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_attribute<'input>(
    prefix: StrSpan<'input>,
    local: StrSpan<'input>,
    value: StrSpan<'input>,
    token_span: StrSpan<'input>,
    loop_detector: &mut LoopDetector,
    text_buffer: &mut TextBuffer,
    state: &mut ParserState<'input>,
    doc: &mut Document<'input>,
) -> Result<(), Error> {
    #[cfg(not(feature = "positions"))]
    let _ = token_span;
    #[cfg(feature = "positions")]
    let pos = token_span.start();

    let value = normalize_attribute(doc.text, value, &state.entities, loop_detector, text_buffer)?;

    if prefix.as_str() == XMLNS {
        // The xmlns namespace MUST NOT be declared as the default namespace.
        if value.as_str() == NS_XMLNS_URI {
            let pos = err_pos_from_qname(doc.text, prefix, local);
            return Err(Error::UnexpectedXmlnsUri(pos));
        }

        let is_xml_ns_uri = value.as_str() == NS_XML_URI;

        // The prefix 'xml' is by definition bound to the namespace name
        // http://www.w3.org/XML/1998/namespace.
        // It MUST NOT be bound to any other namespace name.
        if local.as_str() == NS_XML_PREFIX {
            if !is_xml_ns_uri {
                let pos = err_pos_from_span(doc.text, prefix);
                return Err(Error::InvalidXmlPrefixUri(pos));
            }
        } else {
            // The xml namespace MUST NOT be bound to a non-xml prefix.
            if is_xml_ns_uri {
                let pos = err_pos_from_span(doc.text, prefix);
                return Err(Error::UnexpectedXmlUri(pos));
            }
        }

        // Check for duplicated namespaces.
        if doc
            .namespaces
            .exists(state.namespace_start_idx, Some(local.as_str()))
        {
            let pos = err_pos_from_qname(doc.text, prefix, local);
            return Err(Error::DuplicatedNamespace(local.as_str().to_string(), pos));
        }

        // Xml namespace should not be added to the namespaces.
        if !is_xml_ns_uri {
            doc.namespaces.push_ns(Some(local.as_str()), value)?;
        }
    } else if local.as_str() == XMLNS {
        // The xml namespace MUST NOT be declared as the default namespace.
        if value.as_str() == NS_XML_URI {
            let pos = err_pos_from_span(doc.text, local);
            return Err(Error::UnexpectedXmlUri(pos));
        }

        // The xmlns namespace MUST NOT be declared as the default namespace.
        if value.as_str() == NS_XMLNS_URI {
            let pos = err_pos_from_span(doc.text, local);
            return Err(Error::UnexpectedXmlnsUri(pos));
        }

        doc.namespaces.push_ns(None, value)?;
    } else {
        let value = value.to_storage();
        state.current_attributes.push(TempAttributeData {
            prefix,
            local,
            value,
            #[cfg(feature = "positions")]
            pos,
        });
    }

    Ok(())
}

fn process_element<'input>(
    tag_name: TagNameSpan<'input>,
    end_token: xmlparser::ElementEnd<'input>,
    token_span: StrSpan<'input>,
    state: &mut ParserState<'input>,
    doc: &mut Document<'input>,
) -> Result<(), Error> {
    if tag_name.name.is_empty() {
        // May occur in XML like this:
        // <!DOCTYPE test [ <!ENTITY p '</p>'> ]>
        // <root>&p;</root>

        if let xmlparser::ElementEnd::Close(..) = end_token {
            return Err(Error::UnexpectedEntityCloseTag(err_pos_from_span(
                doc.text, token_span,
            )));
        } else {
            unreachable!("should be already checked by the xmlparser");
        }
    }

    let namespaces = resolve_namespaces(state.namespace_start_idx, state.parent_id, doc);
    state.namespace_start_idx = doc.namespaces.tree_order.len();

    let attributes = resolve_attributes(state, namespaces, doc)?;

    match end_token {
        xmlparser::ElementEnd::Empty => {
            let tag_ns_idx = get_ns_idx_by_prefix(doc, namespaces, tag_name.prefix)?;
            let new_element_id = doc.append(
                NodeKind::Element {
                    tag_name: ExpandedNameIndexed {
                        namespace_idx: tag_ns_idx,
                        local_name: tag_name.name.as_str(),
                    },
                    attributes,
                    namespaces,
                },
                tag_name.span.start()..token_span.end(),
                state,
            )?;
            state.awaiting_subtree.push(new_element_id);
        }
        xmlparser::ElementEnd::Close(prefix, local) => {
            let prefix = prefix.as_str();
            let local = local.as_str();

            let parent_node = &mut doc.nodes[state.parent_id.get_usize()];
            // should never panic as we start with the single prefix of the
            // root node and always push another one when changing the parent
            let parent_prefix = *state.parent_prefixes.last().unwrap();

            #[cfg(feature = "positions")]
            {
                parent_node.range.end = token_span.end();
            }

            if let NodeKind::Element { ref tag_name, .. } = parent_node.kind {
                if prefix != parent_prefix || local != tag_name.local_name {
                    return Err(Error::UnexpectedCloseTag {
                        expected: gen_qname_string(parent_prefix, tag_name.local_name),
                        actual: gen_qname_string(prefix, local),
                        pos: err_pos_from_span(doc.text, token_span),
                    });
                }
            }
            state.awaiting_subtree.push(state.parent_id);

            if let Some(id) = parent_node.parent {
                state.parent_id = id;
                state.parent_prefixes.pop();
                debug_assert!(!state.parent_prefixes.is_empty());
            } else {
                unreachable!("should be already checked by the xmlparser");
            }
        }
        xmlparser::ElementEnd::Open => {
            let tag_ns_idx = get_ns_idx_by_prefix(doc, namespaces, tag_name.prefix)?;
            state.parent_id = doc.append(
                NodeKind::Element {
                    tag_name: ExpandedNameIndexed {
                        namespace_idx: tag_ns_idx,
                        local_name: tag_name.name.as_str(),
                    },
                    attributes,
                    namespaces,
                },
                tag_name.span.start()..token_span.end(),
                state,
            )?;
            state.parent_prefixes.push(tag_name.prefix.as_str());
        }
    }

    Ok(())
}

fn resolve_namespaces(start_idx: usize, parent_id: NodeId, doc: &mut Document) -> ShortRange {
    if let NodeKind::Element { ref namespaces, .. } = doc.nodes[parent_id.get_usize()].kind {
        let parent_ns = *namespaces;
        if start_idx == doc.namespaces.tree_order.len() {
            return parent_ns;
        }

        for i in parent_ns.to_urange() {
            if !doc.namespaces.exists(
                start_idx,
                doc.namespaces.get(doc.namespaces.tree_order[i]).name,
            ) {
                doc.namespaces.push_ref(i);
            }
        }
    }

    (start_idx..doc.namespaces.tree_order.len()).into()
}

fn resolve_attributes<'input>(
    state: &mut ParserState<'input>,
    namespaces: ShortRange,
    doc: &mut Document<'input>,
) -> Result<ShortRange, Error> {
    if state.current_attributes.is_empty() {
        return Ok(ShortRange::new(0, 0));
    }

    if doc.attributes.len() + state.current_attributes.len() >= core::u32::MAX as usize {
        return Err(Error::AttributesLimitReached);
    }

    let start_idx = doc.attributes.len();

    for attr in state.current_attributes.drain(..) {
        let namespace_idx = if attr.prefix.as_str() == NS_XML_PREFIX {
            // The prefix 'xml' is by definition bound to the namespace name
            // http://www.w3.org/XML/1998/namespace. This namespace is added
            // to the document on creation and is always element 0.
            Some(NamespaceIdx(0))
        } else if attr.prefix.is_empty() {
            // 'The namespace name for an unprefixed attribute name
            // always has no value.'
            None
        } else {
            get_ns_idx_by_prefix(doc, namespaces, attr.prefix)?
        };

        let attr_name = ExpandedNameIndexed {
            namespace_idx,
            local_name: attr.local.as_str(),
        };

        // Check for duplicated attributes.
        if doc.attributes[start_idx..]
            .iter()
            .any(|attr| attr.name.as_expanded_name(doc) == attr_name.as_expanded_name(doc))
        {
            let pos = err_pos_from_qname(doc.text, attr.prefix, attr.local);
            return Err(Error::DuplicatedAttribute(attr.local.to_string(), pos));
        }

        doc.attributes.push(AttributeData {
            name: attr_name,
            value: attr.value,
            #[cfg(feature = "positions")]
            pos: attr.pos,
        });
    }

    Ok((start_idx..doc.attributes.len()).into())
}

fn process_text<'input>(
    text: StrSpan<'input>,
    loop_detector: &mut LoopDetector,
    text_buffer: &mut TextBuffer,
    state: &mut ParserState<'input>,
    doc: &mut Document<'input>,
) -> Result<(), Error> {
    // Add text as is if it has only valid characters.
    if !text.as_str().bytes().any(|b| b == b'&' || b == b'\r') {
        append_text(BorrowedText::Input(text.as_str()), text.range(), doc, state)?;
        state.after_text = true;
        return Ok(());
    }

    text_buffer.clear();

    let mut is_as_is = false; // TODO: explain
    let mut stream = Stream::from_substr(doc.text, text.range());
    while !stream.at_end() {
        match parse_next_chunk(&mut stream, &state.entities)? {
            NextChunk::Byte(c) => {
                if is_as_is {
                    text_buffer.push_raw(c);
                    is_as_is = false;
                } else {
                    text_buffer.push_from_text(c, stream.at_end());
                }
            }
            NextChunk::Char(c) => {
                for b in CharToBytes::new(c) {
                    if loop_detector.depth > 0 {
                        text_buffer.push_from_text(b, stream.at_end());
                    } else {
                        // Characters not from entity should be added as is.
                        // Not sure why... At least `lxml` produces the same result.
                        text_buffer.push_raw(b);
                        is_as_is = true;
                    }
                }
            }
            NextChunk::Text(fragment) => {
                is_as_is = false;

                if !text_buffer.is_empty() {
                    append_text(
                        BorrowedText::Temp(text_buffer.to_str()),
                        text.range(),
                        doc,
                        state,
                    )?;
                    state.after_text = true;
                    text_buffer.clear();
                }

                loop_detector.inc_references(&stream)?;
                loop_detector.inc_depth(&stream)?;

                let parser = xmlparser::Tokenizer::from_fragment(doc.text, fragment.range());
                let mut tag_name = TagNameSpan::new_null();
                process_tokens(
                    parser,
                    loop_detector,
                    &mut tag_name,
                    text_buffer,
                    state,
                    doc,
                )?;
                text_buffer.clear();

                loop_detector.dec_depth();
            }
        }
    }

    if !text_buffer.is_empty() {
        append_text(
            BorrowedText::Temp(text_buffer.to_str()),
            text.range(),
            doc,
            state,
        )?;
        state.after_text = true;
        text_buffer.clear();
    }

    Ok(())
}

pub(crate) enum BorrowedText<'input, 'temp> {
    Input(&'input str),
    Temp(&'temp str),
}

impl<'input, 'temp> BorrowedText<'input, 'temp> {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            BorrowedText::Input(text) => text,
            BorrowedText::Temp(text) => text,
        }
    }

    pub(crate) fn to_storage(&self) -> StringStorage<'input> {
        match self {
            BorrowedText::Input(text) => StringStorage::Borrowed(text),
            BorrowedText::Temp(text) => StringStorage::new_owned(*text),
        }
    }
}

// While the whole purpose of CDATA is to indicate to an XML library that this text
// has to be stored as is, carriage return (`\r`) is still has to be replaced with `\n`.
fn process_cdata<'input>(
    text: StrSpan<'input>,
    span: StrSpan<'input>,
    text_buffer: &mut TextBuffer,
    state: &mut ParserState<'input>,
    doc: &mut Document<'input>,
) -> Result<(), Error> {
    // Add text as is if it has only valid characters.
    if !text.as_str().as_bytes().contains(&b'\r') {
        append_text(BorrowedText::Input(text.as_str()), span.range(), doc, state)?;
        state.after_text = true;
        return Ok(());
    }

    text_buffer.clear();

    let count = text.as_str().chars().count();
    for (i, c) in text.as_str().chars().enumerate() {
        for b in CharToBytes::new(c) {
            text_buffer.push_from_text(b, i + 1 == count);
        }
    }

    if !text_buffer.is_empty() {
        append_text(
            BorrowedText::Temp(text_buffer.to_str()),
            text.range(),
            doc,
            state,
        )?;
        state.after_text = true;
        text_buffer.clear();
    }

    Ok(())
}

fn append_text<'input>(
    text: BorrowedText<'input, '_>,
    range: Range<usize>,
    doc: &mut Document<'input>,
    state: &mut ParserState<'input>,
) -> Result<(), Error> {
    if state.after_text {
        // Prepend to a previous text node.
        if let Some(node) = doc.nodes.last_mut() {
            if let NodeKind::Text(ref mut prev_text) = node.kind {
                let text_str = text.as_str();
                let prev_text_str = prev_text.as_str();

                let mut concat_text = String::with_capacity(text_str.len() + prev_text_str.len());
                concat_text.push_str(prev_text_str);
                concat_text.push_str(text_str);
                *prev_text = StringStorage::new_owned(concat_text);
            }
        }
    } else {
        let text = text.to_storage();
        doc.append(NodeKind::Text(text), range, state)?;
    }

    Ok(())
}

enum NextChunk<'a> {
    Byte(u8),
    Char(char),
    Text(StrSpan<'a>),
}

fn parse_next_chunk<'a>(
    stream: &mut Stream<'a>,
    entities: &[Entity<'a>],
) -> Result<NextChunk<'a>, Error> {
    debug_assert!(!stream.at_end());

    // Safe, because we already checked that stream is not at the end.
    // But we have an additional `debug_assert` above just in case.
    let c = stream.curr_byte_unchecked();

    // Check for character/entity references.
    if c == b'&' {
        let start = stream.pos();
        match stream.try_consume_reference() {
            Some(Reference::Char(ch)) => Ok(NextChunk::Char(ch)),
            Some(Reference::Entity(name)) => entities
                .iter()
                .find(|e| e.name == name)
                .map(|e| NextChunk::Text(e.value))
                .ok_or_else(|| {
                    let pos = stream.gen_text_pos_from(start);
                    Error::UnknownEntityReference(name.into(), pos)
                }),
            None => {
                let pos = stream.gen_text_pos_from(start);
                Err(Error::MalformedEntityReference(pos))
            }
        }
    } else {
        stream.advance(1);
        Ok(NextChunk::Byte(c))
    }
}

// https://www.w3.org/TR/REC-xml/#AVNormalize
fn normalize_attribute<'input, 'temp>(
    input: &'input str,
    text: StrSpan<'input>,
    entities: &[Entity],
    loop_detector: &mut LoopDetector,
    buffer: &'temp mut TextBuffer,
) -> Result<BorrowedText<'input, 'temp>, Error> {
    if is_normalization_required(&text) {
        buffer.clear();
        _normalize_attribute(input, text, entities, loop_detector, buffer)?;
        Ok(BorrowedText::Temp(buffer.to_str()))
    } else {
        Ok(BorrowedText::Input(text.as_str()))
    }
}

#[inline]
fn is_normalization_required(text: &StrSpan) -> bool {
    // We assume that `&` indicates an entity or a character reference.
    // But in rare cases it can be just an another character.

    fn check(c: u8) -> bool {
        match c {
            b'&' | b'\t' | b'\n' | b'\r' => true,
            _ => false,
        }
    }

    text.as_str().bytes().any(check)
}

fn _normalize_attribute(
    input: &str,
    text: StrSpan,
    entities: &[Entity],
    loop_detector: &mut LoopDetector,
    buffer: &mut TextBuffer,
) -> Result<(), Error> {
    let mut stream = Stream::from_substr(input, text.range());
    while !stream.at_end() {
        // Safe, because we already checked that the stream is not at the end.
        let c = stream.curr_byte_unchecked();

        if c != b'&' {
            stream.advance(1);
            buffer.push_from_attr(c, stream.curr_byte().ok());
            continue;
        }

        // Check for character/entity references.
        let start = stream.pos();
        match stream.try_consume_reference() {
            Some(Reference::Char(ch)) => {
                for b in CharToBytes::new(ch) {
                    if loop_detector.depth > 0 {
                        // Escaped `<` inside an ENTITY is an error.
                        // Escaped `<` outside an ENTITY is ok.
                        if b == b'<' {
                            return Err(Error::InvalidAttributeValue(
                                stream.gen_text_pos_from(start),
                            ));
                        }

                        buffer.push_from_attr(b, None);
                    } else {
                        // Characters not from entity should be added as is.
                        // Not sure why... At least `lxml` produces the same results.
                        buffer.push_raw(b);
                    }
                }
            }
            Some(Reference::Entity(name)) => match entities.iter().find(|e| e.name == name) {
                Some(entity) => {
                    loop_detector.inc_references(&stream)?;
                    loop_detector.inc_depth(&stream)?;
                    _normalize_attribute(input, entity.value, entities, loop_detector, buffer)?;
                    loop_detector.dec_depth();
                }
                None => {
                    let pos = stream.gen_text_pos_from(start);
                    return Err(Error::UnknownEntityReference(name.into(), pos));
                }
            },
            None => {
                let pos = stream.gen_text_pos_from(start);
                return Err(Error::MalformedEntityReference(pos));
            }
        }
    }

    Ok(())
}

fn get_ns_idx_by_prefix(
    doc: &Document,
    range: ShortRange,
    prefix: StrSpan,
) -> Result<Option<NamespaceIdx>, Error> {
    // Prefix CAN be empty when the default namespace was defined.
    //
    // Example:
    // <e xmlns='http://www.w3.org'/>
    let prefix_opt = if prefix.is_empty() {
        None
    } else {
        Some(prefix.as_str())
    };

    let idx = doc.namespaces.tree_order[range.to_urange()]
        .iter()
        .find(|idx| doc.namespaces.get(**idx).name == prefix_opt);

    match idx {
        Some(idx) => Ok(Some(*idx)),
        None => {
            if !prefix.is_empty() {
                // If an URI was not found and prefix IS NOT empty than
                // we have an unknown namespace.
                //
                // Example:
                // <e random:a='b'/>
                let pos = err_pos_from_span(doc.text, prefix);
                Err(Error::UnknownNamespace(prefix.as_str().to_string(), pos))
            } else {
                // If an URI was not found and prefix IS empty than
                // an element or an attribute doesn't have a namespace.
                //
                // Example:
                // <e a='b'/>
                Ok(None)
            }
        }
    }
}

fn gen_qname_string(prefix: &str, local: &str) -> String {
    if prefix.is_empty() {
        local.to_string()
    } else {
        alloc::format!("{}:{}", prefix, local)
    }
}

fn err_pos_from_span(input: &str, text: StrSpan) -> TextPos {
    Stream::from_substr(input, text.range()).gen_text_pos()
}

fn err_pos_from_qname(input: &str, prefix: StrSpan, local: StrSpan) -> TextPos {
    let err_span = if prefix.is_empty() { local } else { prefix };
    err_pos_from_span(input, err_span)
}

mod internals {
    use alloc::vec::Vec;

    /// Iterate over `char` by `u8`.
    pub struct CharToBytes {
        buf: [u8; 4],
        idx: u8,
    }

    impl CharToBytes {
        #[inline]
        pub fn new(c: char) -> Self {
            let mut buf = [0xFF; 4];
            c.encode_utf8(&mut buf);

            CharToBytes { buf, idx: 0 }
        }
    }

    impl Iterator for CharToBytes {
        type Item = u8;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.idx < 4 {
                let b = self.buf[self.idx as usize];

                if b != 0xFF {
                    self.idx += 1;
                    return Some(b);
                } else {
                    self.idx = 4;
                }
            }

            None
        }
    }

    pub struct TextBuffer {
        buffer: Vec<u8>,
    }

    impl TextBuffer {
        #[inline]
        pub fn new() -> Self {
            TextBuffer {
                buffer: Vec::with_capacity(32),
            }
        }

        #[inline]
        pub fn push_raw(&mut self, c: u8) {
            self.buffer.push(c);
        }

        pub fn push_from_attr(&mut self, mut current: u8, next: Option<u8>) {
            // \r in \r\n should be ignored.
            if current == b'\r' && next == Some(b'\n') {
                return;
            }

            // \n, \r and \t should be converted into spaces.
            current = match current {
                b'\n' | b'\r' | b'\t' => b' ',
                _ => current,
            };

            self.buffer.push(current);
        }

        // Translate \r\n and any \r that is not followed by \n into a single \n character.
        //
        // https://www.w3.org/TR/xml/#sec-line-ends
        pub fn push_from_text(&mut self, c: u8, at_end: bool) {
            if self.buffer.last() == Some(&b'\r') {
                let idx = self.buffer.len() - 1;
                self.buffer[idx] = b'\n';

                if at_end && c == b'\r' {
                    self.buffer.push(b'\n');
                } else if c != b'\n' {
                    self.buffer.push(c);
                }
            } else if at_end && c == b'\r' {
                self.buffer.push(b'\n');
            } else {
                self.buffer.push(c);
            }
        }

        #[inline]
        pub fn clear(&mut self) {
            self.buffer.clear();
        }

        #[inline]
        pub fn is_empty(&self) -> bool {
            self.buffer.is_empty()
        }

        #[inline]
        pub fn to_str(&self) -> &str {
            // `unwrap` is safe, because buffer must contain a valid UTF-8 string.
            core::str::from_utf8(&self.buffer).unwrap()
        }
    }
}

use self::internals::*;
