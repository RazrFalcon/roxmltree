use alloc::string::{String, ToString};
use alloc::{collections::VecDeque, vec, vec::Vec};
use alloc::borrow::Cow;
use core::ops::Range;
use core::mem::take;
use core::fmt;
use memchr::{memchr, memchr2, memchr_iter};

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

    /// Indicates that the [`ParsingOptions::nodes_limit`] was reached,
    /// or that elements are nested deeper than `u32::MAX` levels.
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

    /// Resolving the given entity yielded an error.
    EntityResolver(TextPos, String),

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
            Error::EntityResolver(pos, _) => pos,
            Error::InvalidComment(pos) => pos,
            Error::InvalidCharacterData(pos) => pos,
            Error::UnknownToken(pos) => pos,
            Error::UnexpectedEndOfStream => TextPos::new(1, 1),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
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
                    *expected as char, *actual as char, pos
                )
            }
            Error::InvalidChar2(expected, actual, pos) => {
                write!(
                    f,
                    "expected {} not '{}' at {}",
                    expected, *actual as char, pos
                )
            }
            Error::InvalidString(expected, pos) => {
                write!(f, "expected '{}' at {}", expected, pos)
            }
            Error::InvalidExternalID(pos) => {
                write!(f, "invalid ExternalID at {}", pos)
            }
            Error::EntityResolver(pos, msg) => {
                write!(f, "entity resolver failed at {}: {}", pos, msg)
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
pub struct ParsingOptions<'input> {
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

    /// Function to resolve external entities
    ///
    /// See [`EntityResolver`] for the signature
    /// and the expected behaviour.
    pub entity_resolver: Option<&'input EntityResolver<'input>>,
}

/// Function to resolve external entities
///
/// This function is passed the optional public ID
/// and the mandatory URI of the external entity.
///
/// It is expected to yield the content defining
/// the URI, possibly preceeded by a text declaration.
///
/// If it yields [`None`], the entity is discarded
/// and will yield errors if referenced during parsing.
///
/// Errors must be stringified so they can be propagated
/// via [`Error::EntityResolver`].
pub type EntityResolver<'input> =
    dyn Fn(Option<&str>, &str) -> core::result::Result<Option<&'input str>, String> + 'input;

impl Default for ParsingOptions<'_> {
    fn default() -> Self {
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: u32::MAX,
            entity_resolver: None,
        }
    }
}

impl fmt::Debug for ParsingOptions<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entity_resolver = if self.entity_resolver.is_some() {
            "Some(..)"
        } else {
            "None"
        };

        fmt.debug_struct("ParsingOptions")
            .field("allow_dtd", &self.allow_dtd)
            .field("nodes_limit", &self.nodes_limit)
            .field("entity_resolver", &entity_resolver)
            .finish()
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
    pub fn parse(text: &'input str) -> Result<Self> {
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
    pub fn parse_with_options(text: &'input str, opt: ParsingOptions<'input>) -> Result<Self> {
        parse(text, opt)
    }
}

/// Parses XML with the same semantic checks as `Document::parse_with_options`.
/// Event IDs follow the indexed document's node order; zero is the implicit root.
/// The callback can observe events before a later parse error.
pub fn parse_events<'input>(
    text: &'input str,
    opt: ParsingOptions<'input>,
    mut sink: impl for<'a> FnMut(crate::ValidatedEvent<'a, 'input>),
) -> Result<()> {
    let mut reader = EventReader::new(text, opt)?;
    while let Some(event) = reader.next_event()? {
        sink(event);
    }
    Ok(())
}

/// A pull reader using the same XML validation as indexed and callback parsing.
///
/// Events borrow this reader until the next mutable access. Reaching the end of
/// input proves complete validation only if no earlier read returned an error.
/// Dropping a reader early, or reaching fused exhaustion after an error, does
/// not prove validation.
pub struct EventReader<'input> {
    execution: Execution<'input>,
    current: Option<PendingEvent<'input>>,
    error: Option<Error>,
    failed: bool,
}

impl fmt::Debug for EventReader<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EventReader")
            .field("nodes_seen", &self.execution.context.nodes_seen)
            .field("open_elements", &self.execution.context.parents.len())
            .field("complete", &self.execution.complete)
            .field("failed", &self.failed)
            .finish_non_exhaustive()
    }
}

impl<'input> EventReader<'input> {
    /// Creates a reader without parsing ahead into the input.
    pub fn new(text: &'input str, options: ParsingOptions<'input>) -> Result<Self> {
        Ok(Self {
            execution: Execution::new(text, options, Output::Events)?,
            current: None,
            error: None,
            failed: false,
        })
    }

    /// Reads the next semantically checked event, or the terminal parse error.
    ///
    /// Attribute, namespace and text views remain valid while the event is
    /// borrowed. After an error, subsequent calls return `Ok(None)`; that fused
    /// exhaustion does not indicate successful validation.
    ///
    /// A borrowed event cannot outlive the next mutable read:
    ///
    /// ```compile_fail
    /// let mut reader = roxmltree::EventReader::new("<r/>", Default::default()).unwrap();
    /// let first = reader.next_event().unwrap().unwrap();
    /// let second = reader.next_event();
    /// println!("{:?}", first);
    /// ```
    pub fn next_event(&mut self) -> Result<Option<crate::ValidatedEvent<'_, 'input>>> {
        if let Some(PendingEvent::Node {
            kind: NodeKind::Element { .. },
            ..
        }) = self.current.take()
        {
            self.execution.context.doc.attributes.clear();
        }
        if self.failed {
            return Ok(None);
        }
        loop {
            if let Some(event) = self.execution.context.pending.pop_front() {
                self.current = Some(event);
                break;
            }
            if let Some(error) = self.error.take() {
                self.failed = true;
                return Err(error);
            }
            if self.execution.complete {
                return Ok(None);
            }
            if let Err(error) = self.execution.step() {
                self.error = Some(error);
                self.execution.tasks.clear();
            }
        }
        Ok(self
            .current
            .as_ref()
            .map(|event| event.view(&self.execution.context.doc)))
    }
}

enum PendingEvent<'input> {
    Node {
        id: NodeId,
        parent: NodeId,
        kind: NodeKind<'input>,
        range: Range<usize>,
    },
    End {
        id: NodeId,
        end: usize,
    },
}

impl<'input> PendingEvent<'input> {
    fn view<'a>(&'a self, doc: &'a Document<'input>) -> crate::ValidatedEvent<'a, 'input> {
        use crate::ValidatedEvent;
        match self {
            Self::Node {
                id,
                parent,
                kind,
                range,
            } => match kind {
                NodeKind::Element {
                    tag_name,
                    attributes,
                    namespaces,
                } => ValidatedEvent::ElementStart {
                    id: *id,
                    parent: *parent,
                    tag_name: tag_name.as_expanded_name(doc),
                    attributes: crate::Attributes {
                        doc,
                        attrs: doc.attributes[attributes.to_urange()].iter(),
                    },
                    namespaces: crate::NamespaceIter {
                        doc,
                        namespaces: doc.namespaces.tree_order[namespaces.to_urange()].iter(),
                    },
                    start: range.start,
                },
                NodeKind::Text(text) => ValidatedEvent::Text {
                    id: *id,
                    parent: *parent,
                    text: text.as_str(),
                    range: range.clone(),
                },
                NodeKind::Comment(text) => ValidatedEvent::Comment {
                    id: *id,
                    parent: *parent,
                    text: text.as_str(),
                    range: range.clone(),
                },
                NodeKind::PI(pi) => ValidatedEvent::ProcessingInstruction {
                    id: *id,
                    parent: *parent,
                    pi: *pi,
                    range: range.clone(),
                },
                NodeKind::Root => unreachable!(),
            },
            Self::End { id, end } => ValidatedEvent::ElementEnd { id: *id, end: *end },
        }
    }
}

struct Execution<'input> {
    context: Context<'input>,
    tasks: Vec<Task<'input>>,
    complete: bool,
}

enum Task<'input> {
    Lexical {
        driver: tokenizer::Driver<'input>,
        entity: Option<TagNameSpan<'input>>,
    },
    Text(TextState<'input>),
}

impl<'input> Execution<'input> {
    fn new(text: &'input str, options: ParsingOptions<'input>, output: Output) -> Result<Self> {
        let driver = tokenizer::Driver::document(text, options.allow_dtd);
        Ok(Self {
            context: Context::new(text, options, output)?,
            tasks: vec![Task::Lexical {
                driver,
                entity: None,
            }],
            complete: false,
        })
    }

    fn step(&mut self) -> Result<()> {
        if let Some(mut task) = self.tasks.pop() {
            let (again, child) = match &mut task {
                Task::Lexical { driver, entity } => {
                    let again = driver.step(&mut self.context)?;
                    if !again {
                        if let Some(previous) = entity {
                            self.context.tag_name = *previous;
                            self.context.loop_detector.dec_depth();
                        }
                    }
                    (again, self.context.scheduled.take().map(Task::Text))
                }
                Task::Text(text) => text.step(&mut self.context)?,
            };
            if again {
                self.tasks.push(task);
            }
            if let Some(child) = child {
                self.tasks.push(child);
            }
        } else {
            if !self.context.root_seen {
                return Err(Error::NoRootNode);
            }
            if self.context.parents.len() > 1 {
                return Err(Error::UnclosedRootNode);
            }
            self.context.reset_after_text();
            self.complete = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod pull_backpressure_tests {
    #[test]
    fn creating_a_reader_does_not_build_the_future_index() {
        let reader = super::EventReader::new(
            "<root><first/><future><nested/></future></root>",
            super::ParsingOptions::default(),
        )
        .unwrap();
        assert_eq!(
            reader.execution.context.doc.nodes.len(),
            1,
            "only the implicit root may exist before the first pull"
        );
        assert_eq!(reader.execution.context.nodes_seen, 1);
        assert_eq!(reader.execution.context.doc.attributes.len(), 0);
    }
    #[test]
    fn first_child_pull_leaves_future_source_unread() {
        let xml = "<root><first x='1'/><future><nested/></future></root>";
        let mut reader = super::EventReader::new(xml, super::ParsingOptions::default()).unwrap();
        assert!(
            matches!(reader.next_event().unwrap(), Some(crate::ValidatedEvent::ElementStart { id, .. }) if id.get() == 1)
        );
        assert!(
            matches!(reader.next_event().unwrap(), Some(crate::ValidatedEvent::ElementStart { id, .. }) if id.get() == 2)
        );
        assert_eq!(reader.execution.context.nodes_seen, 3);
        assert_eq!(reader.execution.context.doc.nodes.len(), 1);
        assert_eq!(reader.execution.context.pending.len(), 1); // only first's closing event
        match &reader.execution.tasks[0] {
            super::Task::Lexical { driver, .. } => {
                assert_eq!(driver.stream.pos(), xml.find("<future>").unwrap())
            }
            _ => panic!("the document frame remains lexical"),
        }
    }

    #[test]
    fn markup_entity_expansion_stops_at_the_first_child() {
        let children = "<a x='value'/>".repeat(5000);
        let xml = alloc::format!(
            "<!DOCTYPE root [<!ENTITY many \"{}\">]><root>&many;</root>",
            children
        );
        let mut reader = super::EventReader::new(
            &xml,
            super::ParsingOptions {
                allow_dtd: true,
                ..super::ParsingOptions::default()
            },
        )
        .unwrap();
        assert!(
            matches!(reader.next_event().unwrap(), Some(crate::ValidatedEvent::ElementStart { id, .. }) if id.get() == 1)
        );
        assert!(
            matches!(reader.next_event().unwrap(), Some(crate::ValidatedEvent::ElementStart { id, attributes, .. }) if id.get() == 2 && attributes.len() == 1)
        );
        assert_eq!(reader.execution.context.nodes_seen, 3);
        assert_eq!(reader.execution.context.pending.len(), 1);
        assert_eq!(reader.execution.context.doc.nodes.len(), 1);
        match reader.execution.tasks.last().unwrap() {
            super::Task::Lexical {
                driver,
                entity: Some(_),
            } => assert_eq!(
                driver.stream.pos(),
                xml.find("<a x='value'/>").unwrap() + "<a x='value'/>".len()
            ),
            _ => panic!("the entity continuation remains resumable"),
        }
    }

    #[test]
    fn dropping_during_dtd_does_not_resolve_future_declarations() {
        let calls = core::cell::Cell::new(0);
        let resolver = |_: Option<&str>, _: &str| {
            calls.set(calls.get() + 1);
            Ok(Some("value"))
        };
        let xml = "<!DOCTYPE r [<!--first--><!ENTITY later SYSTEM 'resource'>]><r>&later;</r>";
        let mut reader = super::EventReader::new(
            xml,
            super::ParsingOptions {
                allow_dtd: true,
                entity_resolver: Some(&resolver),
                ..super::ParsingOptions::default()
            },
        )
        .unwrap();
        assert!(matches!(
            reader.next_event().unwrap(),
            Some(crate::ValidatedEvent::Comment { text: "first", .. })
        ));
        assert_eq!(calls.get(), 0);
        drop(reader);
        assert_eq!(calls.get(), 0);
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

enum Output {
    Indexed,
    Events,
}

#[derive(Clone, Copy)]
struct OpenElement<'input> {
    id: NodeId,
    prefix: &'input str,
    local: &'input str,
    namespaces: ShortRange,
}

struct Context<'input> {
    opt: ParsingOptions<'input>,
    namespace_start_idx: usize,
    current_attributes: Vec<TempAttributeData<'input>>,
    awaiting_subtree: Vec<NodeId>,
    parents: Vec<OpenElement<'input>>,
    entities: Vec<Entity<'input>>,
    after_text: Vec<Cow<'input, str>>,
    pending_text: Option<(NodeId, Range<usize>)>,
    nodes_seen: usize,
    attributes_seen: usize,
    root_seen: bool,
    tag_name: TagNameSpan<'input>,
    loop_detector: LoopDetector,
    output: Output,
    // One lexical construct can flush text and emit one tag's start/end.
    // Entity content is stepped separately; future subtree events never queue.
    pending: VecDeque<PendingEvent<'input>>,
    scheduled: Option<TextState<'input>>,
    doc: Document<'input>,
}

impl<'input> Context<'input> {
    fn new(text: &'input str, opt: ParsingOptions<'input>, output: Output) -> Result<Self> {
        let indexed = matches!(output, Output::Indexed);
        let nodes_capacity = if indexed {
            memchr_iter(b'<', text.as_bytes()).count()
        } else {
            1
        };
        let attributes_capacity = if indexed {
            memchr_iter(b'=', text.as_bytes()).count()
        } else {
            16
        };
        let mut doc = Document {
            text,
            nodes: Vec::with_capacity(nodes_capacity),
            attributes: Vec::with_capacity(attributes_capacity),
            namespaces: Namespaces::default(),
        };
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
        Ok(Self {
            opt,
            namespace_start_idx: 1,
            current_attributes: Vec::with_capacity(16),
            entities: Vec::new(),
            awaiting_subtree: Vec::new(),
            parents: vec![OpenElement {
                id: NodeId::new(0),
                prefix: "",
                local: "",
                namespaces: ShortRange::new(0, 0),
            }],
            after_text: Vec::with_capacity(1),
            pending_text: None,
            nodes_seen: 1,
            attributes_seen: 0,
            root_seen: false,
            tag_name: TagNameSpan::new_null(),
            loop_detector: LoopDetector::default(),
            output,
            pending: VecDeque::new(),
            scheduled: None,
            doc,
        })
    }

    fn reserve_node(&mut self) -> Result<NodeId> {
        if self.nodes_seen >= self.opt.nodes_limit as usize {
            return Err(Error::NodesLimitReached);
        }
        let id = NodeId::from(self.nodes_seen);
        self.nodes_seen += 1;
        Ok(id)
    }

    fn append_node(&mut self, kind: NodeKind<'input>, range: Range<usize>) -> Result<NodeId> {
        let id = self.reserve_node()?;
        self.append_node_data(id, kind, range);
        Ok(id)
    }

    fn append_node_data(&mut self, id: NodeId, kind: NodeKind<'input>, range: Range<usize>) {
        let parent = self.parents.last().expect("the document frame remains open").id;
        let element = matches!(kind, NodeKind::Element { .. });
        self.root_seen |= element && parent == NodeId::new(0);
        match &mut self.output {
            Output::Indexed => {
                debug_assert_eq!(id.get_usize(), self.doc.nodes.len());
                self.doc.nodes.push(NodeData {
                    parent: Some(parent),
                    prev_sibling: None,
                    next_subtree: None,
                    last_child: None,
                    kind,
                    #[cfg(feature = "positions")]
                    range,
                });
                let last_child = self.doc.nodes[parent.get_usize()].last_child;
                self.doc.nodes[id.get_usize()].prev_sibling = last_child;
                self.doc.nodes[parent.get_usize()].last_child = Some(id);
                for previous in &self.awaiting_subtree {
                    self.doc.nodes[previous.get_usize()].next_subtree = Some(id);
                }
                self.awaiting_subtree.clear();
                if !element {
                    self.awaiting_subtree.push(id);
                }
            }
            Output::Events => self.pending.push_back(PendingEvent::Node { id, parent, kind, range }),
        }
    }

    fn close_element(&mut self, id: NodeId, end: usize) {
        match &mut self.output {
            Output::Indexed => {
                #[cfg(feature = "positions")]
                {
                    self.doc.nodes[id.get_usize()].range.end = end;
                }
                self.awaiting_subtree.push(id);
            }
            Output::Events => self.pending.push_back(PendingEvent::End { id, end }),
        }
    }

    fn append_text(&mut self, text: Cow<'input, str>, range: Range<usize>) -> Result<()> {
        if self.after_text.is_empty() {
            self.pending_text = Some((self.reserve_node()?, range));
        }
        self.after_text.push(text);
        Ok(())
    }

    fn reset_after_text(&mut self) {
        if self.after_text.is_empty() {
            return;
        }
        let (id, range) = self.pending_text.take().expect("text has a reserved node");
        let text = if self.after_text.len() == 1 {
            match self.after_text.pop().expect("one text chunk") {
                Cow::Borrowed(text) => StringStorage::Borrowed(text),
                Cow::Owned(text) => StringStorage::new_owned(&text),
            }
        } else {
            let text = StringStorage::new_owned(&self.after_text.join(""));
            self.after_text.clear();
            text
        };
        self.append_node_data(id, NodeKind::Text(text), range);
    }
}

fn parse<'input>(text: &'input str, opt: ParsingOptions<'input>) -> Result<Document<'input>> {
    parse_with_output(text, opt, Output::Indexed)
}

fn parse_with_output<'input>(
    text: &'input str,
    opt: ParsingOptions<'input>,
    output: Output,
) -> Result<Document<'input>> {
    let indexed = matches!(output, Output::Indexed);
    let mut execution = Execution::new(text, opt, output)?;
    while !execution.complete {
        execution.step()?;
    }
    let mut doc = execution.context.doc;
    if indexed {
        doc.nodes.shrink_to_fit();
        doc.attributes.shrink_to_fit();
        doc.namespaces.shrink_to_fit();
    }
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

    fn resolve_entity(&mut self, pub_id: Option<&str>, uri: &str) -> core::result::Result<Option<&'input str>, String> {
        match &mut self.opt.entity_resolver {
            Some(entity_resolver) => entity_resolver(pub_id, uri),
            None => Ok(None),
        }
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
            return Err(Error::UnexpectedEntityCloseTag(ctx.doc.text_pos_at(token_range.start)));
        } else {
            unreachable!("should be already checked by the tokenizer");
        }
    }

    let namespaces = ctx.resolve_namespaces();
    ctx.namespace_start_idx = ctx.doc.namespaces.tree_order.len();

    let attributes = resolve_attributes(namespaces, ctx)?;

    match end_token {
        tokenizer::ElementEnd::Empty => {
            let tag_ns_idx = get_ns_idx_by_prefix(namespaces, ctx.tag_name.prefix_pos, ctx.tag_name.prefix, &ctx.doc)?;
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
            ctx.close_element(new_element_id, token_range.end);
        }
        tokenizer::ElementEnd::Close(prefix, local) => {
            let parent = *ctx.parents.last().expect("the document frame remains open");
            if ctx.parents.len() == 1 {
                return Err(Error::UnexpectedEntityCloseTag(ctx.doc.text_pos_at(token_range.start)));
            }
            if prefix != parent.prefix || local != parent.local {
                return Err(Error::UnexpectedCloseTag(
                    gen_qname_string(parent.prefix, parent.local),
                    gen_qname_string(prefix, local),
                    ctx.doc.text_pos_at(token_range.start),
                ));
            }
            ctx.close_element(parent.id, token_range.end);
            ctx.parents.pop();
        }
        tokenizer::ElementEnd::Open => {
            let tag_ns_idx = get_ns_idx_by_prefix(namespaces, ctx.tag_name.prefix_pos, ctx.tag_name.prefix, &ctx.doc)?;
            let id = ctx.append_node(
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
            ctx.parents.push(OpenElement {
                id,
                prefix: ctx.tag_name.prefix,
                local: ctx.tag_name.name,
                namespaces,
            });
        }
    }

    Ok(())
}

impl Context<'_> {
    fn resolve_namespaces(&mut self) -> ShortRange {
        if self.parents.len() > 1 {
            let parent_ns = self.parents.last().expect("an element frame").namespaces;
            if self.namespace_start_idx == self.doc.namespaces.tree_order.len() {
                return parent_ns;
            }

            for i in parent_ns.to_urange() {
                if !self.doc.namespaces.exists(
                    self.namespace_start_idx,
                    self.doc.namespaces.get(self.doc.namespaces.tree_order[i]).name,
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

    if ctx.attributes_seen + ctx.current_attributes.len() >= u32::MAX as usize {
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

        ctx.attributes_seen += 1;
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
    if memchr2(b'&', b'\r', text.as_bytes()).is_none() {
        ctx.append_text(Cow::Borrowed(text), range)?;
    } else {
        ctx.scheduled = Some(TextState {
            stream: Stream::from_substr(ctx.doc.text, range.clone()),
            range,
            buffer: TextBuffer::new(),
            is_as_is: false,
        });
    }
    Ok(())
}

struct TextState<'input> {
    stream: Stream<'input>,
    range: Range<usize>,
    buffer: TextBuffer,
    is_as_is: bool,
}

impl<'input> TextState<'input> {
    fn step(&mut self, ctx: &mut Context<'input>) -> Result<(bool, Option<Task<'input>>)> {
        while !self.stream.at_end() {
            match parse_next_chunk(&mut self.stream, &ctx.entities)? {
                NextChunk::Byte(c) => {
                    if self.is_as_is {
                        self.buffer.push_raw(c);
                        self.is_as_is = false;
                    } else {
                        self.buffer.push_from_text(c, self.stream.at_end());
                    }
                }
                NextChunk::Char(c) => {
                    for byte in CharToBytes::new(c) {
                        if ctx.loop_detector.depth > 0 {
                            self.buffer.push_from_text(byte, self.stream.at_end());
                        } else {
                            self.buffer.push_raw(byte);
                            self.is_as_is = true;
                        }
                    }
                }
                NextChunk::Text(fragment) => {
                    self.is_as_is = false;
                    if !self.buffer.is_empty() {
                        ctx.append_text(Cow::Owned(self.buffer.finish()), self.range.clone())?;
                    }
                    ctx.loop_detector.inc_references(&self.stream)?;
                    ctx.loop_detector.inc_depth(&self.stream)?;
                    let source = if fragment.range().start == 0 {
                        fragment.as_str()
                    } else {
                        ctx.doc.text
                    };
                    let stream = Stream::from_substr(source, fragment.range());
                    let previous = ctx.tag_name;
                    ctx.tag_name = TagNameSpan::new_null();
                    self.buffer.clear();
                    return Ok((
                        true,
                        Some(Task::Lexical {
                            driver: tokenizer::Driver::content(stream),
                            entity: Some(previous),
                        }),
                    ));
                }
            }
        }
        if !self.buffer.is_empty() {
            ctx.append_text(Cow::Owned(self.buffer.finish()), self.range.clone())?;
        }
        Ok((false, None))
    }
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
    if memchr2(b'&', b'\t', text.as_str().as_bytes()).is_some() || memchr2(b'\n', b'\r', text.as_str().as_bytes()).is_some() {
        let mut text_buffer = TextBuffer::new();
        _normalize_attribute(text, &mut text_buffer, ctx)?;
        Ok(StringStorage::new_owned(&text_buffer.finish()))
    } else {
        Ok(StringStorage::Borrowed(text.as_str()))
    }
}

fn _normalize_attribute(text: StrSpan, buffer: &mut TextBuffer, ctx: &mut Context) -> Result<()> {
    let mut stream = Stream::from_substr(ctx.doc.text, text.range());
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
        match stream.consume_reference() {
            Some(Reference::Char(ch)) => {
                for b in CharToBytes::new(ch) {
                    if ctx.loop_detector.depth > 0 {
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
            Some(Reference::Entity(name)) => match ctx.entities.iter().find(|e| e.name == name) {
                Some(entity) => {
                    ctx.loop_detector.inc_references(&stream)?;
                    ctx.loop_detector.inc_depth(&stream)?;
                    _normalize_attribute(entity.value, buffer, ctx)?;
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

    fn push_from_attr(&mut self, mut current: u8, next: Option<u8>) {
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

#[cfg(test)]
mod event_counter_tests {
    use super::*;


    #[test]
    fn discarded_nodes_still_reach_the_configured_limit() {
        let mut context = Context::new("<a/>", ParsingOptions::default(), Output::Events).unwrap();
        context.opt.nodes_limit = 3;
        context.nodes_seen = 3;
        assert_eq!(context.doc.nodes.len(), 1);
        assert_eq!(
            context.append_node(NodeKind::Comment(StringStorage::Borrowed("")), 0..0),
            Err(Error::NodesLimitReached)
        );
    }

    #[test]
    fn discarded_attributes_still_reach_the_total_limit() {
        let mut context = Context::new("<a x='v'/>", ParsingOptions::default(), Output::Events).unwrap();
        context.attributes_seen = u32::MAX as usize - 2;
        let attribute = || TempAttributeData {
            prefix: "",
            local: "x",
            value: StringStorage::Borrowed("v"),
            range: 3..8,
            #[cfg(feature = "positions")]
            qname_len: 1,
            #[cfg(feature = "positions")]
            eq_len: 1,
        };
        context.current_attributes.push(attribute());
        let namespaces = ShortRange::new(0, 0);
        let attributes = resolve_attributes(namespaces, &mut context).unwrap();
        context
            .append_node(
                NodeKind::Element {
                    tag_name: ExpandedNameIndexed {
                        namespace_idx: None,
                        local_name: "a",
                    },
                    attributes,
                    namespaces,
                },
                0..10,
            )
            .unwrap();
        context.pending.clear();
        context.doc.attributes.clear(); // released after the lending start event
        assert!(context.doc.attributes.is_empty());
        assert_eq!(context.attributes_seen, u32::MAX as usize - 1);
        context.current_attributes.push(attribute());
        assert!(matches!(
            resolve_attributes(namespaces, &mut context),
            Err(Error::AttributesLimitReached)
        ));
    }
}
