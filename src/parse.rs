use alloc::borrow::{Cow, Borrow, ToOwned};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use xmlparser::{
    self,
    Reference,
    Stream,
    StrSpan,
    TextPos,
};

use crate::{
    NS_XML_URI,
    NS_XMLNS_URI,
    Attribute,
    Document,
    ExpandedNameOwned,
    Namespaces,
    NodeData,
    NodeId,
    NodeKind,
    PI,
    ShortRange,
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
    UnexpectedCloseTag { expected: String, actual: String, pos: TextPos },

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

    /// The input string should be smaller than 4GiB.
    SizeLimit,

    /// An XML with DTD detected.
    ///
    /// This error will be emitted only when `ParsingOptions::allow_dtd` is set to `false`.
    DtdDetected,

    /// Errors detected by the `xmlparser` crate.
    ParserError(xmlparser::Error),
}

impl Error {
    /// Returns the error position.
    #[inline]
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
            _ => TextPos::new(1, 1)
        }
    }
}

impl From<xmlparser::Error> for Error {
    #[inline]
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
                write!(f, "the 'xml' namespace URI is used for not 'xml' prefix at {}", pos)
            }
            Error::UnexpectedXmlnsUri(pos) => {
                write!(f, "the 'xmlns' URI is used at {}, but it must not be declared", pos)
            }
            Error::InvalidElementNamePrefix(pos) => {
                write!(f, "the 'xmlns' prefix is used at {}, but it must not be", pos)
            }
            Error::DuplicatedNamespace(ref name, pos) => {
                write!(f, "namespace '{}' at {} is already defined", name, pos)
            }
            Error::UnknownNamespace(ref name, pos) => {
                write!(f, "an unknown namespace prefix '{}' at {}", name, pos)
            }
            Error::UnexpectedCloseTag { ref expected, ref actual, pos } => {
                write!(f, "expected '{}' tag, not '{}' at {}", expected, actual, pos)
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
            Error::SizeLimit => {
                write!(f, "the input string should be smaller than 4GiB")
            }
            Error::DtdDetected => {
                write!(f, "XML with DTD detected")
            }
            Error::ParserError(ref err) => {
                write!(f, "{}", err)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    #[inline]
    fn description(&self) -> &str {
        "an XML parsing error"
    }
}


/// Parsing options.
#[derive(Clone, Copy, PartialEq, Debug)]
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
}

impl Default for ParsingOptions {
    fn default() -> Self {
        ParsingOptions {
            allow_dtd: false,
        }
    }
}


struct AttributeData<'input> {
    prefix: StrSpan<'input>,
    local: StrSpan<'input>,
    value: Cow<'input, str>,
    range: ShortRange,
    value_range: ShortRange,
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
        parent_id: NodeId,
        kind: NodeKind<'input>,
        range: ShortRange,
        pd: &mut ParserData,
    ) -> NodeId {
        let new_child_id = NodeId::from(self.nodes.len());

        let appending_element = match kind {
            NodeKind::Element {..} => true,
            _ => false
        };

        self.nodes.push(NodeData {
            parent: Some(parent_id),
            prev_sibling: None,
            next_subtree: None,
            last_child: None,
            kind,
            range,
        });

        let last_child_id = self.nodes[parent_id.get_usize()].last_child;
        self.nodes[new_child_id.get_usize()].prev_sibling = last_child_id;
        self.nodes[parent_id.get_usize()].last_child = Some(new_child_id);

        pd.awaiting_subtree.iter().for_each(|id| {
            self.nodes[id.get_usize()].next_subtree = Some(new_child_id);
        });
        pd.awaiting_subtree.clear();

        if !appending_element {
            pd.awaiting_subtree.push(NodeId::from(self.nodes.len() - 1));
        }

        new_child_id
    }
}

struct Entity<'input>  {
    name: &'input str,
    value: StrSpan<'input>,
}

struct ParserData<'input> {
    opt: ParsingOptions,
    attrs_start_idx: usize,
    ns_start_idx: usize,
    tmp_attrs: Vec<AttributeData<'input>>,
    awaiting_subtree: Vec<NodeId>,
    entities: Vec<Entity<'input>>,
    buffer: TextBuffer,
    after_text: bool,
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
    fn inc_depth(&mut self, s: &Stream) -> Result<(), Error> {
        if self.depth < 10 {
            self.depth += 1;
            Ok(())
        } else {
            Err(Error::EntityReferenceLoop(s.gen_text_pos()))
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
    fn inc_references(&mut self, s: &Stream) -> Result<(), Error> {
        if self.depth == 0 {
            // Allow infinite amount of references at zero depth.
            Ok(())
        } else {
            if self.references == core::u8::MAX {
                return Err(Error::EntityReferenceLoop(s.gen_text_pos()));
            }

            self.references += 1;
            Ok(())
        }
    }
}


fn parse(text: &str, opt: ParsingOptions) -> Result<Document, Error> {
    if text.len() > core::u32::MAX as usize {
        return Err(Error::SizeLimit);
    }

    let mut pd = ParserData {
        opt,
        attrs_start_idx: 0,
        ns_start_idx: 1,
        tmp_attrs: Vec::with_capacity(16),
        entities: Vec::new(),
        awaiting_subtree: Vec::new(),
        buffer: TextBuffer::new(),
        after_text: false,
    };

    // Trying to guess rough nodes and attributes amount.
    let nodes_capacity = text.bytes().filter(|c| *c == b'<').count();
    let attributes_capacity = text.bytes().filter(|c| *c == b'=').count();

    // Init document.
    let mut doc = Document {
        text,
        nodes: Vec::with_capacity(nodes_capacity),
        attrs: Vec::with_capacity(attributes_capacity),
        namespaces: Namespaces(Vec::new()),
    };

    // Add a root node.
    doc.nodes.push(NodeData {
        parent: None,
        prev_sibling: None,
        next_subtree: None,
        last_child: None,
        kind: NodeKind::Root,
        range: (0..text.len()).into(),
    });

    doc.namespaces.push_ns(Some("xml"), Cow::Borrowed(NS_XML_URI));

    let parser = xmlparser::Tokenizer::from(text);
    let parent_id = doc.root().id;
    let mut tag_name = TagNameSpan::new_null();
    process_tokens(parser, parent_id, &mut LoopDetector::default(),
                   &mut tag_name, &mut pd, &mut doc)?;

    if !doc.root().children().any(|n| n.is_element()) {
        return Err(Error::NoRootNode);
    }

    doc.nodes.shrink_to_fit();
    doc.attrs.shrink_to_fit();
    doc.namespaces.0.shrink_to_fit();

    Ok(doc)
}

fn process_tokens<'input>(
    parser: xmlparser::Tokenizer<'input>,
    mut parent_id: NodeId,
    loop_detector: &mut LoopDetector,
    tag_name: &mut TagNameSpan<'input>,
    pd: &mut ParserData<'input>,
    doc: &mut Document<'input>,
) -> Result<(), Error> {
    for token in parser {
        let token = token?;
        match token {
            xmlparser::Token::ProcessingInstruction { target, content, span } => {
                let pi = NodeKind::PI(PI {
                    target: target.as_str(),
                    value: content.map(|v| v.as_str()),
                });
                doc.append(parent_id, pi, span.range().into(), pd);
            }
            xmlparser::Token::Comment { text, span } => {
                doc.append(parent_id, NodeKind::Comment(text.as_str()), span.range().into(), pd);
            }
            xmlparser::Token::Text { text } => {
                process_text(text, parent_id, loop_detector, pd, doc)?;
            }
            xmlparser::Token::Cdata { text, span } => {
                let cow_str = Cow::Borrowed(text.as_str());
                append_text(cow_str, parent_id, span.range().into(), pd.after_text, doc, pd);
                pd.after_text = true;
            }
            xmlparser::Token::ElementStart { prefix, local, span } => {
                if prefix.as_str() == "xmlns" {
                    let pos = err_pos_from_span(doc.text, prefix);
                    return Err(Error::InvalidElementNamePrefix(pos));
                }

                *tag_name = TagNameSpan::new(prefix, local, span);
            }
            xmlparser::Token::Attribute { prefix, local, value, span } => {
                process_attribute(prefix, local, value, span, loop_detector, pd, doc)?;
            }
            xmlparser::Token::ElementEnd { end, span } => {
                process_element(*tag_name, end, span, &mut parent_id, pd, doc)?;
            }
            xmlparser::Token::DtdStart { .. } => {
                if !pd.opt.allow_dtd {
                    return Err(Error::DtdDetected);
                }
            }
            xmlparser::Token::EntityDeclaration { name, definition, .. } => {
                if let xmlparser::EntityDefinition::EntityValue(value) = definition {
                    pd.entities.push(Entity { name: name.as_str(), value });
                }
            }
            _ => {}
        }

        match token {
            xmlparser::Token::ProcessingInstruction { .. } |
            xmlparser::Token::Comment { .. } |
            xmlparser::Token::ElementStart { .. } |
            xmlparser::Token::ElementEnd { .. } => {
                pd.after_text = false;
            }
            _ => {}
        }
    }

    Ok(())
}

fn process_attribute<'input>(
    prefix: StrSpan<'input>,
    local: StrSpan<'input>,
    value: StrSpan<'input>,
    token_span: StrSpan<'input>,
    loop_detector: &mut LoopDetector,
    pd: &mut ParserData<'input>,
    doc: &mut Document<'input>,
) -> Result<(), Error> {
    let range = token_span.range().into();
    let value_range = value.range().into();
    let value = normalize_attribute(doc.text, value, &pd.entities, loop_detector, &mut pd.buffer)?;

    if prefix.as_str() == "xmlns" {
        // The xmlns namespace MUST NOT be declared as the default namespace.
        if value == NS_XMLNS_URI {
            let pos = err_pos_from_qname(doc.text, prefix, local);
            return Err(Error::UnexpectedXmlnsUri(pos));
        }

        let is_xml_ns_uri = value == NS_XML_URI;

        // The prefix 'xml' is by definition bound to the namespace name
        // http://www.w3.org/XML/1998/namespace.
        // It MUST NOT be bound to any other namespace name.
        if local.as_str() == "xml" {
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
        if doc.namespaces.exists(pd.ns_start_idx, Some(local.as_str())) {
            let pos = err_pos_from_qname(doc.text, prefix, local);
            return Err(Error::DuplicatedNamespace(local.as_str().to_string(), pos));
        }

        // Xml namespace should not be added to the namespaces.
        if !is_xml_ns_uri {
            doc.namespaces.push_ns(Some(local.as_str()), value);
        }
    } else if local.as_str() == "xmlns" {
        // The xml namespace MUST NOT be declared as the default namespace.
        if value == NS_XML_URI {
            let pos = err_pos_from_span(doc.text, local);
            return Err(Error::UnexpectedXmlUri(pos));
        }

        // The xmlns namespace MUST NOT be declared as the default namespace.
        if value == NS_XMLNS_URI {
            let pos = err_pos_from_span(doc.text, local);
            return Err(Error::UnexpectedXmlnsUri(pos));
        }

        doc.namespaces.push_ns(None, value);
    } else {
        pd.tmp_attrs.push(AttributeData {
            prefix, local, value, range, value_range
        });
    }

    Ok(())
}

fn process_element<'input>(
    tag_name: TagNameSpan<'input>,
    end_token: xmlparser::ElementEnd<'input>,
    token_span: StrSpan<'input>,
    parent_id: &mut NodeId,
    pd: &mut ParserData<'input>,
    doc: &mut Document<'input>,
) -> Result<(), Error> {
    if tag_name.name.is_empty() {
        // May occur in XML like this:
        // <!DOCTYPE test [ <!ENTITY p '</p>'> ]>
        // <root>&p;</root>

        if let xmlparser::ElementEnd::Close(..) = end_token {
            return Err(Error::UnexpectedEntityCloseTag(err_pos_from_span(doc.text, token_span)));
        } else {
            unreachable!("should be already checked by the xmlparser");
        }
    }

    let namespaces = resolve_namespaces(pd.ns_start_idx, *parent_id, doc);
    pd.ns_start_idx = doc.namespaces.len();

    let attributes = resolve_attributes(pd.attrs_start_idx, namespaces.clone(),
                                        &mut pd.tmp_attrs, doc)?;
    pd.attrs_start_idx = doc.attrs.len();
    pd.tmp_attrs.clear();

    match end_token {
        xmlparser::ElementEnd::Empty => {
            let tag_ns_uri = get_ns_by_prefix(doc, namespaces.clone(), tag_name.prefix)?;
            let new_element_id = doc.append(*parent_id,
                NodeKind::Element {
                    tag_name: ExpandedNameOwned {
                        ns: tag_ns_uri,
                        prefix: tag_name.prefix.as_str(),
                        name: tag_name.name.as_str(),
                    },
                    attributes,
                    namespaces,
                },
                (tag_name.span.start()..token_span.end()).into(),
                pd
            );
            pd.awaiting_subtree.push(new_element_id);
        }
        xmlparser::ElementEnd::Close(prefix, local) => {
            let prefix = prefix.as_str();
            let local = local.as_str();

            doc.nodes[parent_id.get_usize()].range.end = token_span.end() as u32;
            if let NodeKind::Element { ref tag_name, .. } = doc.nodes[parent_id.get_usize()].kind {
                if prefix != tag_name.prefix || local != tag_name.name {
                    return Err(Error::UnexpectedCloseTag {
                        expected: gen_qname_string(tag_name.prefix, tag_name.name),
                        actual: gen_qname_string(prefix, local),
                        pos: err_pos_from_span(doc.text, token_span),
                    });
                }
            }
            pd.awaiting_subtree.push(*parent_id);

            if let Some(id) = doc.nodes[parent_id.get_usize()].parent {
                *parent_id = id;
            } else {
                unreachable!("should be already checked by the xmlparser");
            }
        }
        xmlparser::ElementEnd::Open => {
            let tag_ns_uri = get_ns_by_prefix(doc, namespaces.clone(), tag_name.prefix)?;
            *parent_id = doc.append(*parent_id,
                NodeKind::Element {
                    tag_name: ExpandedNameOwned {
                        ns: tag_ns_uri,
                        prefix: tag_name.prefix.as_str(),
                        name: tag_name.name.as_str(),
                    },
                    attributes,
                    namespaces,
                },
                (tag_name.span.start()..token_span.end()).into(),
                pd
            );
        }
    }

    Ok(())
}

fn resolve_namespaces(
    start_idx: usize,
    parent_id: NodeId,
    doc: &mut Document,
) -> ShortRange {
    if let NodeKind::Element { ref namespaces, .. } = doc.nodes[parent_id.get_usize()].kind {
        let parent_ns = namespaces.clone();
        if start_idx == doc.namespaces.len() {
            return parent_ns;
        }

        for i in parent_ns.to_urange() {
            if !doc.namespaces.exists(start_idx, doc.namespaces[i].name) {
                let v = doc.namespaces[i].clone();
                doc.namespaces.0.push(v);
            }
        }
    }

    (start_idx..doc.namespaces.len()).into()
}

fn resolve_attributes<'input>(
    start_idx: usize,
    namespaces: ShortRange,
    tmp_attrs: &mut [AttributeData<'input>],
    doc: &mut Document<'input>,
) -> Result<ShortRange, Error> {
    if tmp_attrs.is_empty() {
        return Ok(ShortRange::new(0, 0));
    }

    for attr in tmp_attrs {
        let ns = if attr.prefix.as_str() == "xml" {
            // The prefix 'xml' is by definition bound to the namespace name
            // http://www.w3.org/XML/1998/namespace.
            Some(Cow::Borrowed(NS_XML_URI))
        } else if attr.prefix.is_empty() {
            // 'The namespace name for an unprefixed attribute name
            // always has no value.'
            None
        } else {
            get_ns_by_prefix(doc, namespaces.clone(), attr.prefix)?
        };

        // We do not store attribute prefixes since `ExpandedNameOwned::prefix`
        // is used only for closing tags matching during parsing.
        let attr_name = ExpandedNameOwned { ns, prefix: "", name: attr.local.as_str() };

        // Check for duplicated attributes.
        if doc.attrs[start_idx..].iter().any(|attr| attr.name == attr_name) {
            let pos = err_pos_from_qname(doc.text, attr.prefix, attr.local);
            return Err(Error::DuplicatedAttribute(attr.local.to_string(), pos));
        }

        doc.attrs.push(Attribute {
            name: attr_name,
            // Takes a value from a slice without consuming the slice.
            value: core::mem::replace(&mut attr.value, Cow::Borrowed("")),
            range: attr.range.clone(),
            value_range: attr.value_range.clone(),
        });
    }

    Ok((start_idx..doc.attrs.len()).into())
}

fn process_text<'input>(
    text: StrSpan<'input>,
    parent_id: NodeId,
    loop_detector: &mut LoopDetector,
    pd: &mut ParserData<'input>,
    doc: &mut Document<'input>,
) -> Result<(), Error> {
    // Add text as is if it has only valid characters.
    if !text.as_str().bytes().any(|b| b == b'&' || b == b'\r') {
        append_text(Cow::Borrowed(text.as_str()), parent_id, text.range().into(), pd.after_text, doc, pd);
        pd.after_text = true;
        return Ok(());
    }

    pd.buffer.clear();

    let mut is_as_is = false; // TODO: explain
    let mut s = Stream::from_substr(doc.text, text.range());
    while !s.at_end() {
        match parse_next_chunk(&mut s, &pd.entities)? {
            NextChunk::Byte(c) => {
                if is_as_is {
                    pd.buffer.push_raw(c);
                    is_as_is = false;
                } else {
                    pd.buffer.push_from_text(c, s.at_end());
                }
            }
            NextChunk::Char(c) => {
                for b in CharToBytes::new(c) {
                    if loop_detector.depth > 0 {
                        pd.buffer.push_from_text(b, s.at_end());
                    } else {
                        // Characters not from entity should be added as is.
                        // Not sure why... At least `lxml` produces the same result.
                        pd.buffer.push_raw(b);
                        is_as_is = true;
                    }
                }
            }
            NextChunk::Text(fragment) => {
                is_as_is = false;

                if !pd.buffer.is_empty() {
                    let cow_text = Cow::Owned(pd.buffer.to_str().to_owned());
                    append_text(cow_text, parent_id, text.range().into(), pd.after_text, doc, pd);
                    pd.after_text = true;
                    pd.buffer.clear();
                }

                loop_detector.inc_references(&s)?;
                loop_detector.inc_depth(&s)?;

                let parser = xmlparser::Tokenizer::from_fragment(doc.text, fragment.range());
                let mut tag_name = TagNameSpan::new_null();
                process_tokens(parser, parent_id, loop_detector, &mut tag_name, pd, doc)?;
                pd.buffer.clear();

                loop_detector.dec_depth();
            }
        }
    }

    if !pd.buffer.is_empty() {
        let cow_text = Cow::Owned(pd.buffer.to_str().to_owned());
        append_text(cow_text, parent_id, text.range().into(), pd.after_text, doc, pd);
        pd.after_text = true;
        pd.buffer.clear();
    }

    Ok(())
}

fn append_text<'input>(
    text: Cow<'input, str>,
    parent_id: NodeId,
    range: ShortRange,
    after_text: bool,
    doc: &mut Document<'input>,
    pd: &mut ParserData<'input>
) {
    if after_text {
        // Prepend to a previous text node.
        if let Some(node) = doc.nodes.iter_mut().last() {
            if let NodeKind::Text(ref mut prev_text) = node.kind {
                match *prev_text {
                    Cow::Borrowed(..) => {
                        *prev_text = Cow::Owned((*prev_text).to_string() + text.borrow());
                    }
                    Cow::Owned(ref mut s) => {
                        s.push_str(text.borrow());
                    }
                }
            }
        }
    } else {
        doc.append(parent_id, NodeKind::Text(text), range, pd);
    }
}

enum NextChunk<'a> {
    Byte(u8),
    Char(char),
    Text(StrSpan<'a>),
}

fn parse_next_chunk<'a>(
    s: &mut Stream<'a>,
    entities: &[Entity<'a>],
) -> Result<NextChunk<'a>, Error> {
    debug_assert!(!s.at_end());

    // Safe, because we already checked that stream is not at the end.
    // But we have an additional `debug_assert` above just in case.
    let c = s.curr_byte_unchecked();

    // Check for character/entity references.
    if c == b'&' {
        let start = s.pos();
        match s.try_consume_reference() {
            Some(Reference::Char(ch)) => {
                Ok(NextChunk::Char(ch))
            }
            Some(Reference::Entity(name)) => {
                match entities.iter().find(|e| e.name == name) {
                    Some(entity) => {
                        Ok(NextChunk::Text(entity.value))
                    }
                    None => {
                        let pos = s.gen_text_pos_from(start);
                        Err(Error::UnknownEntityReference(name.into(), pos))
                    }
                }
            }
            None => {
                let pos = s.gen_text_pos_from(start);
                Err(Error::MalformedEntityReference(pos))
            }
        }
    } else {
        s.advance(1);
        Ok(NextChunk::Byte(c))
    }
}

// https://www.w3.org/TR/REC-xml/#AVNormalize
fn normalize_attribute<'input>(
    input: &'input str,
    text: StrSpan<'input>,
    entities: &[Entity],
    loop_detector: &mut LoopDetector,
    buffer: &mut TextBuffer,
) -> Result<Cow<'input, str>, Error> {
    if is_normalization_required(&text) {
        buffer.clear();
        _normalize_attribute(input, text, entities, loop_detector, buffer)?;
        Ok(Cow::Owned(buffer.to_str().to_owned()))
    } else {
        Ok(Cow::Borrowed(text.as_str()))
    }
}

#[inline]
fn is_normalization_required(text: &StrSpan) -> bool {
    // We assume that `&` indicates an entity or a character reference.
    // But in rare cases it can be just an another character.

    fn check(c: u8) -> bool {
        match c {
              b'&'
            | b'\t'
            | b'\n'
            | b'\r' => true,
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
    let mut s = Stream::from_substr(input, text.range());
    while !s.at_end() {
        // Safe, because we already checked that the stream is not at the end.
        let c = s.curr_byte_unchecked();

        if c != b'&' {
            s.advance(1);
            buffer.push_from_attr(c, s.curr_byte().ok());
            continue;
        }

        // Check for character/entity references.
        let start = s.pos();
        match s.try_consume_reference() {
            Some(Reference::Char(ch)) => {
                for b in CharToBytes::new(ch) {
                    if loop_detector.depth > 0 {
                        // Escaped `<` inside an ENTITY is an error.
                        // Escaped `<` outside an ENTITY is ok.
                        if b == b'<' {
                            return Err(Error::InvalidAttributeValue(s.gen_text_pos_from(start)));
                        }

                        buffer.push_from_attr(b, None);
                    } else {
                        // Characters not from entity should be added as is.
                        // Not sure why... At least `lxml` produces the same results.
                        buffer.push_raw(b);
                    }
                }
            }
            Some(Reference::Entity(name)) => {
                match entities.iter().find(|e| e.name == name) {
                    Some(entity) => {
                        loop_detector.inc_references(&s)?;
                        loop_detector.inc_depth(&s)?;
                        _normalize_attribute(input, entity.value, entities, loop_detector, buffer)?;
                        loop_detector.dec_depth();
                    }
                    None => {
                        let pos = s.gen_text_pos_from(start);
                        return Err(Error::UnknownEntityReference(name.into(), pos));
                    }
                }
            }
            None => {
                let pos = s.gen_text_pos_from(start);
                return Err(Error::MalformedEntityReference(pos));
            }
        }
    }

    Ok(())
}

fn get_ns_by_prefix<'input>(
    doc: &Document<'input>,
    range: ShortRange,
    prefix: StrSpan,
) -> Result<Option<Cow<'input, str>>, Error> {
    // Prefix CAN be empty when the default namespace was defined.
    //
    // Example:
    // <e xmlns='http://www.w3.org'/>
    let prefix_opt = if prefix.is_empty() { None } else { Some(prefix.as_str()) };

    let uri = doc.namespaces[range.to_urange()].iter()
        .find(|ns| ns.name == prefix_opt)
        .map(|ns| ns.uri.clone());

    match uri {
        Some(v) => Ok(Some(v)),
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

#[inline]
fn gen_qname_string(prefix: &str, local: &str) -> String {
    if prefix.is_empty() {
        local.to_string()
    } else {
        alloc::format!("{}:{}", prefix, local)
    }
}

#[inline]
fn err_pos_from_span(input: &str, text: StrSpan) -> TextPos {
    Stream::from_substr(input, text.range()).gen_text_pos()
}

#[inline]
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

            CharToBytes {
                buf,
                idx: 0,
            }
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
        buf: Vec<u8>,
    }

    impl TextBuffer {
        #[inline]
        pub fn new() -> Self {
            TextBuffer {
                buf: Vec::with_capacity(32),
            }
        }

        #[inline]
        pub fn push_raw(&mut self, c: u8) {
            self.buf.push(c);
        }

        pub fn push_from_attr(&mut self, mut c: u8, c2: Option<u8>) {
            // \r in \r\n should be ignored.
            if c == b'\r' && c2 == Some(b'\n') {
                return;
            }

            // \n, \r and \t should be converted into spaces.
            c = match c {
                b'\n' | b'\r' | b'\t' => b' ',
                _ => c,
            };

            self.buf.push(c);
        }

        // Translate \r\n and any \r that is not followed by \n into a single \n character.
        //
        // https://www.w3.org/TR/xml/#sec-line-ends
        pub fn push_from_text(&mut self, c: u8, at_end: bool) {
            if self.buf.last() == Some(&b'\r') {
                let idx = self.buf.len() - 1;
                self.buf[idx] = b'\n';

                if at_end && c == b'\r' {
                    self.buf.push(b'\n');
                } else if c != b'\n' {
                    self.buf.push(c);
                }
            } else if at_end && c == b'\r' {
                self.buf.push(b'\n');
            } else {
                self.buf.push(c);
            }
        }

        #[inline]
        pub fn clear(&mut self) {
            self.buf.clear();
        }

        #[inline]
        pub fn is_empty(&self) -> bool {
            self.buf.is_empty()
        }

        #[inline]
        pub fn to_str(&self) -> &str {
            // `unwrap` is safe, because buffer must contain a valid UTF-8 string.
            core::str::from_utf8(&self.buf).unwrap()
        }
    }
}

use self::internals::*;
