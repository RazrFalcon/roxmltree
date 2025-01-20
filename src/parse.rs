use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};
use alloc::borrow::Cow;
use core::ops::Range;
use core::mem::take;
use memchr::{memchr, memchr2, memchr3, memchr_iter};

use crate::{
    AttributeData, Document, ExpandedNameIndexed, NamespaceIdx, Namespaces, NodeData, NodeId,
    NodeKind, ShortRange, StringStorage, TextPos, NS_XMLNS_URI, NS_XML_PREFIX, NS_XML_URI, PI,
    XMLNS,
};

use crate::tokenizer::{self, Reference, StrSpan, Stream};

type Result<T> = core::result::Result<T, Error>;

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
    ///
    /// expected, actual, position
    UnexpectedCloseTag(String, String, TextPos),

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

    /// An XML document can have only one XML declaration
    /// and it must be at the start of the document.
    UnexpectedDeclaration(TextPos),

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

    /// An invalid name.
    InvalidName(TextPos),

    /// A non-XML character has occurred.
    ///
    /// Valid characters are: <https://www.w3.org/TR/xml/#char32>
    NonXmlChar(char, TextPos),

    /// An invalid/unexpected character.
    ///
    /// expected, actual, position
    InvalidChar(u8, u8, TextPos),

    /// An invalid/unexpected character.
    ///
    /// expected, actual, position
    InvalidChar2(&'static str, u8, TextPos),

    /// An unexpected string.
    ///
    /// Contains what string was expected.
    InvalidString(&'static str, TextPos),

    /// An invalid ExternalID in the DTD.
    InvalidExternalID(TextPos),

    /// A comment cannot contain `--` or end with `-`.
    InvalidComment(TextPos),

    /// A Character Data node contains an invalid data.
    ///
    /// Currently, only `]]>` is not allowed.
    InvalidCharacterData(TextPos),

    /// An unknown token.
    UnknownToken(TextPos),

    /// The steam ended earlier than we expected.
    ///
    /// Should only appear on invalid input data.
    UnexpectedEndOfStream,
}

impl Error {
    /// Returns the error position.
    pub fn pos(&self) -> TextPos {
        match *self {
            Error::InvalidXmlPrefixUri(pos) => pos,
            Error::UnexpectedXmlUri(pos) => pos,
            Error::UnexpectedXmlnsUri(pos) => pos,
            Error::InvalidElementNamePrefix(pos) => pos,
            Error::DuplicatedNamespace(_, pos) => pos,
            Error::UnknownNamespace(_, pos) => pos,
            Error::UnexpectedCloseTag(_, _, pos) => pos,
            Error::UnexpectedEntityCloseTag(pos) => pos,
            Error::UnknownEntityReference(_, pos) => pos,
            Error::MalformedEntityReference(pos) => pos,
            Error::EntityReferenceLoop(pos) => pos,
            Error::InvalidAttributeValue(pos) => pos,
            Error::DuplicatedAttribute(_, pos) => pos,
            Error::NoRootNode => TextPos::new(1, 1),
            Error::UnclosedRootNode => TextPos::new(1, 1),
            Error::UnexpectedDeclaration(pos) => pos,
            Error::DtdDetected => TextPos::new(1, 1),
            Error::NodesLimitReached => TextPos::new(1, 1),
            Error::AttributesLimitReached => TextPos::new(1, 1),
            Error::NamespacesLimitReached => TextPos::new(1, 1),
            Error::InvalidName(pos) => pos,
            Error::NonXmlChar(_, pos) => pos,
            Error::InvalidChar(_, _, pos) => pos,
            Error::InvalidChar2(_, _, pos) => pos,
            Error::InvalidString(_, pos) => pos,
            Error::InvalidExternalID(pos) => pos,
            Error::InvalidComment(pos) => pos,
            Error::InvalidCharacterData(pos) => pos,
            Error::UnknownToken(pos) => pos,
            Error::UnexpectedEndOfStream => TextPos::new(1, 1),
        }
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
            Error::UnexpectedCloseTag(ref expected, ref actual, pos) => {
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
            Error::UnexpectedDeclaration(pos) => {
                write!(f, "unexpected XML declaration at {}", pos)
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
            Error::InvalidName(pos) => {
                write!(f, "invalid name token at {}", pos)
            }
            Error::NonXmlChar(c, pos) => {
                write!(f, "a non-XML character {:?} found at {}", c, pos)
            }
            Error::InvalidChar(expected, actual, pos) => {
                write!(
                    f,
                    "expected '{}' not '{}' at {}",
                    expected as char, actual as char, pos
                )
            }
            Error::InvalidChar2(expected, actual, pos) => {
                write!(
                    f,
                    "expected {} not '{}' at {}",
                    expected, actual as char, pos
                )
            }
            Error::InvalidString(expected, pos) => {
                write!(f, "expected '{}' at {}", expected, pos)
            }
            Error::InvalidExternalID(pos) => {
                write!(f, "invalid ExternalID at {}", pos)
            }
            Error::InvalidComment(pos) => {
                write!(f, "comment at {} contains '--'", pos)
            }
            Error::InvalidCharacterData(pos) => {
                write!(f, "']]>' at {} is not allowed inside a character data", pos)
            }
            Error::UnknownToken(pos) => {
                write!(f, "unknown token at {}", pos)
            }
            Error::UnexpectedEndOfStream => {
                write!(f, "unexpected end of stream")
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

impl Default for ParsingOptions {
    fn default() -> Self {
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: u32::MAX,
        }
    }
}

struct TempAttributeData<'input> {
    prefix: &'input str,
    local: &'input str,
    value: StringStorage<'input>,
    range: Range<usize>,
    #[cfg(feature = "positions")]
    qname_len: u16,
    #[cfg(feature = "positions")]
    eq_len: u8,
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
    pub fn parse(text: &str) -> Result<Document> {
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
    pub fn parse_with_options(text: &str, opt: ParsingOptions) -> Result<Document> {
        parse(text, opt)
    }
}

struct Entity<'input> {
    name: &'input str,
    value: StrSpan<'input>,
}

#[derive(Clone, Copy)]
struct TagNameSpan<'input> {
    prefix: &'input str,
    name: &'input str,
    pos: usize,
    prefix_pos: usize,
}

impl<'input> TagNameSpan<'input> {
    #[inline]
    fn new_null() -> Self {
        Self {
            prefix: "",
            name: "",
            pos: 0,
            prefix_pos: 0,
        }
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
    fn inc_depth(&mut self, stream: &Stream) -> Result<()> {
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
    fn inc_references(&mut self, stream: &Stream) -> Result<()> {
        if self.depth == 0 {
            // Allow infinite amount of references at zero depth.
            Ok(())
        } else {
            if self.references == u8::MAX {
                return Err(Error::EntityReferenceLoop(stream.gen_text_pos()));
            }

            self.references += 1;
            Ok(())
        }
    }
}

struct Context<'input> {
    opt: ParsingOptions,
    namespace_start_idx: usize,
    current_attributes: Vec<TempAttributeData<'input>>,
    awaiting_subtree: Vec<NodeId>,
    parent_prefixes: Vec<&'input str>,
    entities: Vec<Entity<'input>>,
    after_text: Vec<Cow<'input, str>>,
    parent_id: NodeId,
    tag_name: TagNameSpan<'input>,
    loop_detector: LoopDetector,
    doc: Document<'input>,
}

impl<'input> Context<'input> {
    fn append_node(&mut self, kind: NodeKind<'input>, range: Range<usize>) -> Result<NodeId> {
        if self.doc.nodes.len() >= self.opt.nodes_limit as usize {
            return Err(Error::NodesLimitReached);
        }

        #[cfg(not(feature = "positions"))]
        let _ = range;

        let new_child_id = NodeId::from(self.doc.nodes.len());

        let appending_element = matches!(kind, NodeKind::Element { .. });
        self.doc.nodes.push(NodeData {
            parent: Some(self.parent_id),
            prev_sibling: None,
            next_subtree: None,
            last_child: None,
            kind,
            #[cfg(feature = "positions")]
            range,
        });

        let last_child_id = self.doc.nodes[self.parent_id.get_usize()].last_child;
        self.doc.nodes[new_child_id.get_usize()].prev_sibling = last_child_id;
        self.doc.nodes[self.parent_id.get_usize()].last_child = Some(new_child_id);

        for id in &self.awaiting_subtree {
            self.doc.nodes[id.get_usize()].next_subtree = Some(new_child_id);
        }
        self.awaiting_subtree.clear();

        if !appending_element {
            self.awaiting_subtree
                .push(NodeId::from(self.doc.nodes.len() - 1));
        }

        Ok(new_child_id)
    }

    fn append_text(
        &mut self,
        text: Cow<'input, str>,
        range: Range<usize>,
    ) -> Result<()> {
        if self.after_text.is_empty() {
            let text = match &text {
                Cow::Borrowed(text) => StringStorage::Borrowed(text),
                Cow::Owned(text) => StringStorage::new_owned(text.as_str()),
            };

            self.append_node(NodeKind::Text(text), range)?;
        }

        self.after_text.push(text);
        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn merge_text(&mut self) {
        let node = &mut self.doc.nodes.last_mut().unwrap();

        let text = match &mut node.kind {
            NodeKind::Text(text) => text,
            _ => unreachable!(),
        };

        *text = StringStorage::new_owned(&self.after_text.join(""));
    }

    #[inline]
    fn reset_after_text(&mut self) {
        if self.after_text.is_empty() {
            return;
        }

        if self.after_text.len() > 1 {
            self.merge_text();
        }

        self.after_text.clear();
    }
}

fn parse(text: &str, opt: ParsingOptions) -> Result<Document> {
    // Trying to guess rough nodes and attributes amount.
    let nodes_capacity = memchr_iter(b'<', text.as_bytes()).count();
    let attributes_capacity = memchr_iter(b'=', text.as_bytes()).count();

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
        .push_ns(Some(NS_XML_PREFIX), StringStorage::Borrowed(NS_XML_URI))?;

    let mut ctx = Context {
        opt,
        namespace_start_idx: 1,
        current_attributes: Vec::with_capacity(16),
        entities: Vec::new(),
        awaiting_subtree: Vec::new(),
        parent_prefixes: vec![""],
        after_text: Vec::with_capacity(1),
        parent_id: NodeId::new(0),
        tag_name: TagNameSpan::new_null(),
        loop_detector: LoopDetector::default(),
        doc,
    };

    tokenizer::parse(text, opt.allow_dtd, &mut ctx)?;

    let mut doc = ctx.doc;
    if !doc.root().children().any(|n| n.is_element()) {
        return Err(Error::NoRootNode);
    }

    if ctx.parent_prefixes.len() > 1 {
        return Err(Error::UnclosedRootNode);
    }

    doc.nodes.shrink_to_fit();
    doc.attributes.shrink_to_fit();
    doc.namespaces.shrink_to_fit();

    Ok(doc)
}

impl<'input> tokenizer::XmlEvents<'input> for Context<'input> {
    #[inline(always)]
    fn token(&mut self, token: tokenizer::Token<'input>) -> Result<()> {
        match token {
            tokenizer::Token::ProcessingInstruction(target, value, range) => {
                self.reset_after_text();
                let pi = NodeKind::PI(PI { target, value });
                self.append_node(pi, range)?;
            }
            tokenizer::Token::Comment(text, range) => {
                self.reset_after_text();
                self.append_node(NodeKind::Comment(StringStorage::Borrowed(text)), range)?;
            }
            tokenizer::Token::EntityDeclaration(name, definition) => {
                self.entities.push(Entity {
                    name,
                    value: definition,
                });
            }
            tokenizer::Token::ElementStart(prefix, local, start) => {
                self.reset_after_text();

                if prefix == XMLNS {
                    let pos = self.doc.text_pos_at(start + 1);
                    return Err(Error::InvalidElementNamePrefix(pos));
                }

                self.tag_name = TagNameSpan {
                    prefix,
                    name: local,
                    pos: start,
                    prefix_pos: start + 1,
                };
            }
            tokenizer::Token::Attribute(range, qname_len, eq_len, prefix, local, value) => {
                process_attribute(range, qname_len, eq_len, prefix, local, value, self)?;
            }
            tokenizer::Token::ElementEnd(end, range) => {
                self.reset_after_text();
                process_element(end, range, self)?;
            }
            tokenizer::Token::Text(text, range) => {
                process_text(text, range, self)?;
            }
            tokenizer::Token::Cdata(text, range) => {
                process_cdata(text, range, self)?;
            }
        }

        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn process_attribute<'input>(
    range: Range<usize>,
    qname_len: u16,
    eq_len: u8,
    prefix: &'input str,
    local: &'input str,
    value: StrSpan<'input>,
    ctx: &mut Context<'input>,
) -> Result<()> {
    let value = normalize_attribute(value, ctx)?;

    if prefix == XMLNS {
        // The xmlns namespace MUST NOT be declared as the default namespace.
        if value.as_str() == NS_XMLNS_URI {
            let pos = ctx.doc.text_pos_at(range.start);
            return Err(Error::UnexpectedXmlnsUri(pos));
        }

        let is_xml_ns_uri = value.as_str() == NS_XML_URI;

        // The prefix 'xml' is by definition bound to the namespace name
        // http://www.w3.org/XML/1998/namespace.
        // It MUST NOT be bound to any other namespace name.
        if local == NS_XML_PREFIX {
            if !is_xml_ns_uri {
                let pos = ctx.doc.text_pos_at(range.start);
                return Err(Error::InvalidXmlPrefixUri(pos));
            }
        } else {
            // The xml namespace MUST NOT be bound to a non-xml prefix.
            if is_xml_ns_uri {
                let pos = ctx.doc.text_pos_at(range.start);
                return Err(Error::UnexpectedXmlUri(pos));
            }
        }

        // Check for duplicated namespaces.
        if ctx
            .doc
            .namespaces
            .exists(ctx.namespace_start_idx, Some(local))
        {
            let pos = ctx.doc.text_pos_at(range.start);
            return Err(Error::DuplicatedNamespace(local.to_string(), pos));
        }

        // Xml namespace should not be added to the namespaces.
        if !is_xml_ns_uri {
            ctx.doc.namespaces.push_ns(Some(local), value)?;
        }
    } else if local == XMLNS {
        // The xml namespace MUST NOT be declared as the default namespace.
        if value.as_str() == NS_XML_URI {
            let pos = ctx.doc.text_pos_at(range.start);
            return Err(Error::UnexpectedXmlUri(pos));
        }

        // The xmlns namespace MUST NOT be declared as the default namespace.
        if value.as_str() == NS_XMLNS_URI {
            let pos = ctx.doc.text_pos_at(range.start);
            return Err(Error::UnexpectedXmlnsUri(pos));
        }

        ctx.doc.namespaces.push_ns(None, value)?;
    } else {
        #[cfg(not(feature = "positions"))]
        let _ = (qname_len, eq_len);

        ctx.current_attributes.push(TempAttributeData {
            prefix,
            local,
            value,
            range,
            #[cfg(feature = "positions")]
            qname_len,
            #[cfg(feature = "positions")]
            eq_len,
        });
    }

    Ok(())
}

fn process_element<'input>(
    end_token: tokenizer::ElementEnd<'input>,
    token_range: Range<usize>,
    ctx: &mut Context<'input>,
) -> Result<()> {
    if ctx.tag_name.name.is_empty() {
        // May occur in XML like this:
        // <!DOCTYPE test [ <!ENTITY p '</p>'> ]>
        // <root>&p;</root>

        if let tokenizer::ElementEnd::Close(..) = end_token {
            return Err(Error::UnexpectedEntityCloseTag(
                ctx.doc.text_pos_at(token_range.start),
            ));
        } else {
            unreachable!("should be already checked by the tokenizer");
        }
    }

    let namespaces = ctx.resolve_namespaces();
    ctx.namespace_start_idx = ctx.doc.namespaces.tree_order.len();

    let attributes = resolve_attributes(namespaces, ctx)?;

    match end_token {
        tokenizer::ElementEnd::Empty => {
            let tag_ns_idx = get_ns_idx_by_prefix(
                namespaces,
                ctx.tag_name.prefix_pos,
                ctx.tag_name.prefix,
                &ctx.doc,
            )?;
            let new_element_id = ctx.append_node(
                NodeKind::Element {
                    tag_name: ExpandedNameIndexed {
                        namespace_idx: tag_ns_idx,
                        local_name: ctx.tag_name.name,
                    },
                    attributes,
                    namespaces,
                },
                ctx.tag_name.pos..token_range.end,
            )?;
            ctx.awaiting_subtree.push(new_element_id);
        }
        tokenizer::ElementEnd::Close(prefix, local) => {
            let parent_node = &mut ctx.doc.nodes[ctx.parent_id.get_usize()];
            // should never panic as we start with the single prefix of the
            // root node and always push another one when changing the parent
            let parent_prefix = *ctx.parent_prefixes.last().unwrap();

            #[cfg(feature = "positions")]
            {
                parent_node.range.end = token_range.end;
            }

            if let NodeKind::Element { ref tag_name, .. } = parent_node.kind {
                if prefix != parent_prefix || local != tag_name.local_name {
                    return Err(Error::UnexpectedCloseTag(
                        gen_qname_string(parent_prefix, tag_name.local_name),
                        gen_qname_string(prefix, local),
                        ctx.doc.text_pos_at(token_range.start),
                    ));
                }
            }
            ctx.awaiting_subtree.push(ctx.parent_id);

            if let Some(id) = parent_node.parent {
                ctx.parent_id = id;
                ctx.parent_prefixes.pop();
                debug_assert!(!ctx.parent_prefixes.is_empty());
            } else {
                // May occur in XML like this:
                // <!DOCTYPE test [ <!ENTITY p '<p></p></p>'> ]>
                // <p>&p;&p;

                return Err(Error::UnexpectedEntityCloseTag(
                    ctx.doc.text_pos_at(token_range.start),
                ));
            }
        }
        tokenizer::ElementEnd::Open => {
            let tag_ns_idx = get_ns_idx_by_prefix(
                namespaces,
                ctx.tag_name.prefix_pos,
                ctx.tag_name.prefix,
                &ctx.doc,
            )?;
            ctx.parent_id = ctx.append_node(
                NodeKind::Element {
                    tag_name: ExpandedNameIndexed {
                        namespace_idx: tag_ns_idx,
                        local_name: ctx.tag_name.name,
                    },
                    attributes,
                    namespaces,
                },
                ctx.tag_name.pos..token_range.end,
            )?;
            ctx.parent_prefixes.push(ctx.tag_name.prefix);
        }
    }

    Ok(())
}

impl Context<'_> {
    fn resolve_namespaces(&mut self) -> ShortRange {
        if let NodeKind::Element { ref namespaces, .. } =
            self.doc.nodes[self.parent_id.get_usize()].kind
        {
            let parent_ns = *namespaces;
            if self.namespace_start_idx == self.doc.namespaces.tree_order.len() {
                return parent_ns;
            }

            for i in parent_ns.to_urange() {
                if !self.doc.namespaces.exists(
                    self.namespace_start_idx,
                    self.doc
                        .namespaces
                        .get(self.doc.namespaces.tree_order[i])
                        .name,
                ) {
                    self.doc.namespaces.push_ref(i);
                }
            }
        }

        (self.namespace_start_idx..self.doc.namespaces.tree_order.len()).into()
    }
}

fn resolve_attributes(namespaces: ShortRange, ctx: &mut Context) -> Result<ShortRange> {
    if ctx.current_attributes.is_empty() {
        return Ok(ShortRange::new(0, 0));
    }

    if ctx.doc.attributes.len() + ctx.current_attributes.len() >= u32::MAX as usize {
        return Err(Error::AttributesLimitReached);
    }

    let start_idx = ctx.doc.attributes.len();

    for attr in ctx.current_attributes.drain(..) {
        let namespace_idx = if attr.prefix == NS_XML_PREFIX {
            // The prefix 'xml' is by definition bound to the namespace name
            // http://www.w3.org/XML/1998/namespace. This namespace is added
            // to the document on creation and is always element 0.
            Some(NamespaceIdx(0))
        } else if attr.prefix.is_empty() {
            // 'The namespace name for an unprefixed attribute name
            // always has no value.'
            None
        } else {
            get_ns_idx_by_prefix(namespaces, attr.range.start, attr.prefix, &ctx.doc)?
        };

        let attr_name = ExpandedNameIndexed {
            namespace_idx,
            local_name: attr.local,
        };

        // Check for duplicated attributes.
        if ctx.doc.attributes[start_idx..].iter().any(|attr| {
            attr.name.as_expanded_name(&ctx.doc) == attr_name.as_expanded_name(&ctx.doc)
        }) {
            let pos = ctx.doc.text_pos_at(attr.range.start);
            return Err(Error::DuplicatedAttribute(attr.local.to_string(), pos));
        }

        ctx.doc.attributes.push(AttributeData {
            name: attr_name,
            value: attr.value,
            #[cfg(feature = "positions")]
            range: attr.range,
            #[cfg(feature = "positions")]
            qname_len: attr.qname_len,
            #[cfg(feature = "positions")]
            eq_len: attr.eq_len,
        });
    }

    Ok((start_idx..ctx.doc.attributes.len()).into())
}

fn process_text<'input>(
    text: &'input str,
    range: Range<usize>,
    ctx: &mut Context<'input>,
) -> Result<()> {
    // Add text as is if it has only valid characters.
    if memchr2(b'&', b'\r', text.as_bytes()).is_none() {
        ctx.append_text(Cow::Borrowed(text), range)?;
        return Ok(());
    }

    let mut text_buffer = TextBuffer::new();
    let mut is_as_is = false; // TODO: explain
    let mut stream = Stream::from_substr(ctx.doc.text, range.clone());
    while !stream.at_end() {
        match parse_next_chunk(&mut stream, &ctx.entities)? {
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
                    if ctx.loop_detector.depth > 0 {
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
                    ctx.append_text(Cow::Owned(text_buffer.finish()), range.clone())?;
                }

                ctx.loop_detector.inc_references(&stream)?;
                ctx.loop_detector.inc_depth(&stream)?;

                let mut stream = Stream::from_substr(ctx.doc.text, fragment.range());
                let prev_tag_name = ctx.tag_name;
                ctx.tag_name = TagNameSpan::new_null();
                tokenizer::parse_content(&mut stream, ctx)?;
                ctx.tag_name = prev_tag_name;
                text_buffer.clear();

                ctx.loop_detector.dec_depth();
            }
        }
    }

    if !text_buffer.is_empty() {
        ctx.append_text(Cow::Owned(text_buffer.finish()), range)?;
    }

    Ok(())
}

// While the whole purpose of CDATA is to indicate to an XML library that this text
// has to be stored as is, carriage return (`\r`) is still has to be replaced with `\n`.
fn process_cdata<'input>(
    mut text: &'input str,
    range: Range<usize>,
    ctx: &mut Context<'input>,
) -> Result<()> {
    let mut pos = memchr(b'\r', text.as_bytes());

    // Add text as is if it has only valid characters.
    if pos.is_none() {
        ctx.append_text(Cow::Borrowed(text), range)?;
        return Ok(());
    }

    let mut buf = String::new();

    while let Some(pos1) = pos {
        let (line, rest) = text.split_at(pos1);

        buf.push_str(line);
        buf.push('\n');

        text = if rest.as_bytes().get(1) == Some(&b'\n') {
            &rest[2..]
        } else {
            &rest[1..]
        };

        pos = memchr(b'\r', text.as_bytes());
    }

    buf.push_str(text);

    ctx.append_text(Cow::Owned(buf), range)?;
    Ok(())
}

enum NextChunk<'a> {
    Byte(u8),
    Char(char),
    Text(StrSpan<'a>),
}

fn parse_next_chunk<'a>(stream: &mut Stream<'a>, entities: &[Entity<'a>]) -> Result<NextChunk<'a>> {
    debug_assert!(!stream.at_end());

    // Safe, because we already checked that stream is not at the end.
    // But we have an additional `debug_assert` above just in case.
    let c = stream.curr_byte_unchecked();

    // Check for character/entity references.
    if c == b'&' {
        let start = stream.pos();
        match stream.consume_reference() {
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
fn normalize_attribute<'input>(
    text: StrSpan<'input>,
    ctx: &mut Context<'input>,
) -> Result<StringStorage<'input>> {
    // We assume that `&` indicates an entity or a character reference.
    // But in rare cases it can be just an another character.
    if memchr(b'&', text.as_str().as_bytes()).is_some() || memchr3(b'\t', b'\r', b'\n', text.as_str().as_bytes()).is_some() {
        let mut buf = String::new();
        _normalize_attribute(text, &mut buf, ctx)?;
        Ok(StringStorage::new_owned(&buf))
    } else {
        Ok(StringStorage::Borrowed(text.as_str()))
    }
}

fn _normalize_attribute(text: StrSpan, buf: &mut String, ctx: &mut Context) -> Result<()> {
    let mut stream = Stream::from_substr(ctx.doc.text, text.range());

    while let Some(mut pos) = memchr(b'&', stream.as_str().as_bytes()) {
        while let Some(pos1) = memchr3(b'\t', b'\r', b'\n', &stream.as_str().as_bytes()[..pos]) {
            let (before, after) = stream.as_str().split_at(pos1);

            buf.push_str(before);
            buf.push(' ');

            let skip = if after.starts_with("\r\n") {
                2
            } else {
                1
            };

            stream.advance(pos1 + skip);
            pos -= pos1 + skip;
        }

        buf.push_str(&stream.as_str()[..pos]);
        stream.advance(pos);

        // Check for character/entity references.
        let start = stream.pos();
        match stream.consume_reference() {
            Some(Reference::Char(ch)) => {
                if ctx.loop_detector.depth > 0 {
                    // Escaped `<` inside an ENTITY is an error.
                    // Escaped `<` outside an ENTITY is ok.
                    if ch == '<' {
                        return Err(Error::InvalidAttributeValue(
                            stream.gen_text_pos_from(start),
                        ));
                    }

                    if matches!(ch, '\t' | '\r' | '\n') {
                        buf.push(' ');
                    } else {
                        buf.push(ch);
                    }
                } else {
                    // Characters not from entity should be added as is.
                    // Not sure why... At least `lxml` produces the same results.
                    buf.push(ch);
                }
            }
            Some(Reference::Entity(name)) => match ctx.entities.iter().find(|e| e.name == name) {
                Some(entity) => {
                    ctx.loop_detector.inc_references(&stream)?;
                    ctx.loop_detector.inc_depth(&stream)?;
                    _normalize_attribute(entity.value, buf, ctx)?;
                    ctx.loop_detector.dec_depth();
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

    while let Some(pos) = memchr3(b'\t', b'\r', b'\n', stream.as_str().as_bytes()) {
        let (before, after) = stream.as_str().split_at(pos);

        buf.push_str(before);
        buf.push(' ');

        let skip = if after.starts_with("\r\n") {
            2
        } else {
            1
        };

        stream.advance(pos + skip);
    }

    buf.push_str(stream.as_str());
    Ok(())
}

fn get_ns_idx_by_prefix(
    namespaces: ShortRange,
    prefix_pos: usize,
    prefix: &str,
    doc: &Document<'_>,
) -> Result<Option<NamespaceIdx>> {
    // Prefix CAN be empty when the default namespace was defined.
    //
    // Example:
    // <e xmlns='http://www.w3.org'/>
    let prefix_opt = if prefix.is_empty() {
        None
    } else {
        Some(prefix)
    };

    let idx = doc.namespaces.tree_order[namespaces.to_urange()]
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
                let pos = doc.text_pos_at(prefix_pos);
                Err(Error::UnknownNamespace(prefix.to_string(), pos))
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

/// Iterate over `char` by `u8`.
struct CharToBytes {
    buf: [u8; 4],
    idx: u8,
}

impl CharToBytes {
    #[inline]
    fn new(c: char) -> Self {
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

struct TextBuffer {
    buffer: Vec<u8>,
}

impl TextBuffer {
    #[inline]
    fn new() -> Self {
        TextBuffer {
            buffer: Vec::with_capacity(32),
        }
    }

    #[inline]
    fn push_raw(&mut self, c: u8) {
        self.buffer.push(c);
    }

    // Translate \r\n and any \r that is not followed by \n into a single \n character.
    //
    // https://www.w3.org/TR/xml/#sec-line-ends
    fn push_from_text(&mut self, c: u8, at_end: bool) {
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
    fn clear(&mut self) {
        self.buffer.clear();
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[inline]
    fn finish(&mut self) -> String {
        // `unwrap` is safe, because buffer must contain a valid UTF-8 string.
        String::from_utf8(take(&mut self.buffer)).unwrap()
    }
}
