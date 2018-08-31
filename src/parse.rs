use std::fmt;
use std::error;
use std::str;
use std::io::Write;
use std::mem;

use xmlparser::{
    self,
    TextPos,
    Reference,
    Stream,
    StrSpan,
};

use {
    NS_XMLNS_URI,
    NS_XML_URI,
    Attribute,
    Document,
    ExpandedNameOwned,
    Namespaces,
    Node,
    NodeData,
    NodeId,
    NodeKind,
    PI,
};


/// A list of all possible errors.
#[derive(Debug)]
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

    /// Nested entity references are not supported.
    ///
    /// Example:
    /// ```xml
    /// <!DOCTYPE test [
    ///     <!ENTITY a '&b;'>
    ///     <!ENTITY b 'text'>
    /// ]>
    /// <e a='&a;'/>
    /// ```
    NestedEntityReference(TextPos),

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

    /// Errors detected by the `xmlparser` crate.
    ParserError(xmlparser::Error),
}

impl From<xmlparser::Error> for Error {
    fn from(e: xmlparser::Error) -> Self {
        Error::ParserError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
            Error::UnexpectedCloseTag { ref expected, ref actual, pos } => {
                write!(f, "expected '{}' tag, not '{}' at {}", expected, actual, pos)
            }
            Error::UnexpectedEntityCloseTag(pos) => {
                write!(f, "unexpected close tag at {}", pos)
            }
            Error::UnknownEntityReference(ref name, pos) => {
                write!(f, "unknown entity reference '{}' at {}", name, pos)
            }
            Error::NestedEntityReference(pos) => {
                write!(f, "nested entity reference detected at {}, it's not supported", pos)
            }
            Error::DuplicatedAttribute(ref name, pos) => {
                write!(f, "attribute '{}' at {} is already defined", name, pos)
            }
            Error::NoRootNode => {
                write!(f, "the document does not have a root node")
            }
            Error::ParserError(ref err) => {
                write!(f, "{}", err)
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "an XML parsing error"
    }
}


struct AttributeData<'d> {
    prefix: StrSpan<'d>,
    prefix_str: &'d str,
    local: StrSpan<'d>,
    local_str: &'d str,
    value_pos: usize,
    value: String,
}

impl<'d> Document<'d> {
    /// Parses the input XML string.
    ///
    /// We do not support `&[u8]` or `Reader` because the input must be an already allocated
    /// UTF-8 string.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = roxmltree::Document::parse("<e/>").unwrap();
    /// assert_eq!(doc.descendants().count(), 2); // root node + `e` element node
    /// ```
    pub fn parse(text: &str) -> Result<Document, Error> {
        parse(text)
    }

    fn append(&mut self, parent_id: NodeId, kind: NodeKind<'d>, orig_pos: usize) -> NodeId {
        let new_child_id = NodeId(self.nodes.len());
        self.nodes.push(NodeData {
            parent: Some(parent_id),
            prev_sibling: None,
            next_sibling: None,
            children: None,
            kind,
            orig_pos,
        });

        let last_child_id = self.nodes[parent_id.0].children.map(|(_, id)| id);
        self.nodes[new_child_id.0].prev_sibling = last_child_id;

        if let Some(id) = last_child_id {
            self.nodes[id.0].next_sibling = Some(new_child_id);
        }

        self.nodes[parent_id.0].children = Some(
            if let Some((first_child_id, _)) = self.nodes[parent_id.0].children {
                (first_child_id, new_child_id)
            } else {
                (new_child_id, new_child_id)
            }
        );

        new_child_id
    }

    fn get(&self, id: NodeId) -> Node {
        Node { id, d: &self.nodes[id.0], doc: self }
    }
}

struct ParserData<'d> {
    attrs_start_idx: usize,
    ns_start_idx: usize,
    tmp_attrs: Vec<AttributeData<'d>>,
    entities: Vec<(&'d str, StrSpan<'d>)>,
    u_buffer: Vec<u8>,
    prev_node_type: Option<xmlparser::Token<'d>>,
}

#[derive(Clone, Copy)]
struct TagNameSpan<'d> {
    prefix: StrSpan<'d>,
    name: StrSpan<'d>,
}

impl<'d> TagNameSpan<'d> {
    fn new(prefix: StrSpan<'d>, name: StrSpan<'d>) -> Self {
        Self { prefix, name }
    }
}

fn parse(text: &str) -> Result<Document, Error> {
    let mut pd = ParserData {
        attrs_start_idx: 0,
        ns_start_idx: 2,
        tmp_attrs: Vec::new(),
        entities: Vec::new(),
        u_buffer: Vec::with_capacity(32),
        prev_node_type: None,
    };

    let nodes_capacity = text.bytes().filter(|c| *c == b'<').count();
    let attributes_capacity = text.bytes().filter(|c| *c == b'=').count();

    let mut doc = Document {
        text,
        nodes: Vec::with_capacity(nodes_capacity),
        attrs: Vec::with_capacity(attributes_capacity),
        namespaces: Namespaces(Vec::new()),
    };

    doc.nodes.push(NodeData {
        parent: None,
        prev_sibling: None,
        next_sibling: None,
        children: None,
        kind: NodeKind::Root,
        orig_pos: 0,
    });

    doc.namespaces.push_ns("", String::new());
    doc.namespaces.push_ns("xml", NS_XML_URI.to_string());

    let parser = xmlparser::Tokenizer::from(text);
    let parent_id = doc.root().id;
    let mut tag_name = TagNameSpan::new(StrSpan::from(""), StrSpan::from(""));
    process_tokens(parser, false, parent_id, &mut tag_name, &mut pd, &mut doc)?;

    if !doc.root().children().any(|n| n.is_element()) {
        return Err(Error::NoRootNode);
    }

    Ok(doc)
}

fn process_tokens<'d>(
    parser: xmlparser::Tokenizer<'d>,
    nested: bool,
    mut parent_id: NodeId,
    tag_name: &mut TagNameSpan<'d>,
    pd: &mut ParserData<'d>,
    doc: &mut Document<'d>,
) -> Result<(), Error> {
    for token in parser {
        let token = token?;
        match token {
            xmlparser::Token::ProcessingInstruction(target, content) => {
                doc.append(parent_id,
                    NodeKind::PI(PI {
                        target: target.to_str(),
                        value: content.map(|v| v.to_str()),
                    }),
                    target.start() - 2, // jump before '<?'
                );
            }
            xmlparser::Token::Comment(text) => {
                let orig_pos = text.start() - 4; // jump before '<!--'
                doc.append(parent_id, NodeKind::Comment(text.to_str()), orig_pos);
            }
            xmlparser::Token::Text(text) |
            xmlparser::Token::Whitespaces(text) => {
                process_text(text, parent_id, nested, pd, doc)?;
            }
            xmlparser::Token::Cdata(text) => {
                process_cdata(text, parent_id, pd, doc);
            }
            xmlparser::Token::ElementStart(prefix, local) => {
                let prefix_str = prefix.to_str();

                if prefix_str == "xmlns" {
                    let pos = err_pos_from_span(prefix);
                    return Err(Error::InvalidElementNamePrefix(pos));
                }

                *tag_name = TagNameSpan::new(prefix, local);
            }
            xmlparser::Token::Attribute((prefix, local), value) => {
                process_attribute(prefix, local, value, pd, doc)?;
            }
            xmlparser::Token::ElementEnd(end) => {
                process_element(*tag_name, end, &mut parent_id, pd, doc)?;
            }
            xmlparser::Token::EntityDeclaration(name, value) => {
                if let xmlparser::EntityDefinition::EntityValue(value) = value {
                    pd.entities.push((name.to_str(), value));
                }
            }
            _ => {}
        }

        pd.prev_node_type = Some(token);
    }

    Ok(())
}

fn process_attribute<'d>(
    prefix: StrSpan<'d>,
    local: StrSpan<'d>,
    value: StrSpan<'d>,
    pd: &mut ParserData<'d>,
    doc: &mut Document<'d>,
) -> Result<(), Error> {
    let prefix_str = prefix.to_str();
    let local_str = local.to_str();
    let orig_pos = value.start();
    let value = normalize_attribute(value, false, &pd.entities, &mut pd.u_buffer)?;

    if prefix_str == "xmlns" {
        // The xmlns namespace MUST NOT be declared as the default namespace.
        if value == NS_XMLNS_URI {
            let pos = err_pos_from_qname(prefix, local);
            return Err(Error::UnexpectedXmlnsUri(pos));
        }

        let is_xml_ns_uri = value == NS_XML_URI;

        // The prefix 'xml' is by definition bound to the namespace name
        // http://www.w3.org/XML/1998/namespace.
        // It MUST NOT be bound to any other namespace name.
        if local_str == "xml" {
            if !is_xml_ns_uri {
                let pos = err_pos_from_span(prefix);
                return Err(Error::InvalidXmlPrefixUri(pos));
            }
        } else {
            // The xml namespace MUST NOT be bound to a non-xml prefix.
            if is_xml_ns_uri {
                let pos = err_pos_from_span(prefix);
                return Err(Error::UnexpectedXmlUri(pos));
            }
        }

        // Check for duplicated namespaces.
        if doc.namespaces[pd.ns_start_idx..].iter().any(|attr| attr.name == local_str) {
            let pos = err_pos_from_qname(prefix, local);
            return Err(Error::DuplicatedNamespace(local_str.to_string(), pos));
        }

        // Xml namespace should not be added to the namespaces.
        if !is_xml_ns_uri {
            doc.namespaces.push_ns(local_str, value);
        }
    } else if local_str == "xmlns" {
        // The xml namespace MUST NOT be declared as the default namespace.
        if value == NS_XML_URI {
            let pos = err_pos_from_span(local);
            return Err(Error::UnexpectedXmlUri(pos));
        }

        // The xmlns namespace MUST NOT be declared as the default namespace.
        if value == NS_XMLNS_URI {
            let pos = err_pos_from_span(local);
            return Err(Error::UnexpectedXmlnsUri(pos));
        }

        doc.namespaces.push_ns("", value);
    } else {
        pd.tmp_attrs.push(AttributeData { prefix, prefix_str, local, local_str,
                                          value_pos: orig_pos, value });
    }

    Ok(())
}

fn process_element<'d>(
    tag_name: TagNameSpan<'d>,
    end_token: xmlparser::ElementEnd<'d>,
    parent_id: &mut NodeId,
    pd: &mut ParserData<'d>,
    doc: &mut Document<'d>,
) -> Result<(), Error> {
    if tag_name.name.is_empty() {
        // May occur in XML like this:
        // <!DOCTYPE test [ <!ENTITY p '</p>'> ]>
        // <root>&p;</root>

        if let xmlparser::ElementEnd::Close(prefix, local) = end_token {
            return Err(Error::UnexpectedEntityCloseTag(err_pos_from_tag_name(prefix, local, true)));
        } else {
            unreachable!("should be already checked by the xmlparser");
        }
    }

    // Resolve namespaces.
    let mut tmp_parent_id = parent_id.0;
    while tmp_parent_id != 0 {
        let curr_id = tmp_parent_id;
        tmp_parent_id = match doc.nodes[tmp_parent_id].parent {
            Some(id) => id.0,
            None => 0,
        };

        let ns_range = match doc.nodes[curr_id].kind {
            NodeKind::Element { ref namespaces, .. } => namespaces.clone(),
            _ => continue,
        };

        for i in ns_range {
            if !doc.namespaces.exists(pd.ns_start_idx, doc.namespaces[i].name) {
                let v = doc.namespaces[i].clone();
                doc.namespaces.0.push(v);
            }
        }
    }

    let mut namespaces = 0..0;
    if pd.ns_start_idx != doc.namespaces.len() {
        namespaces = pd.ns_start_idx..doc.namespaces.len();
        pd.ns_start_idx = doc.namespaces.len();
    }


    // Resolve attributes.
    let mut attributes = 0..0;
    if !pd.tmp_attrs.is_empty() {
        for attr in &mut pd.tmp_attrs {
            let ns = if attr.prefix_str == "xml" {
                // The prefix 'xml' is by definition bound to the namespace name
                // http://www.w3.org/XML/1998/namespace.
                doc.namespaces.xml_uri()
            } else if attr.prefix_str.is_empty() {
                // 'The namespace name for an unprefixed attribute name
                // always has no value.'
                doc.namespaces.null_uri()
            } else {
                doc.namespaces.get_by_prefix(namespaces.clone(), attr.prefix_str)
            };

            let attr_name = ExpandedNameOwned { ns, name: attr.local_str };

            // Check for duplicated attributes.
            if doc.attrs[pd.attrs_start_idx..].iter().any(|attr| attr.name == attr_name) {
                let pos = err_pos_from_qname(attr.prefix, attr.local);
                return Err(Error::DuplicatedAttribute(attr.local_str.to_string(), pos));
            }

            let attr_pos = if attr.prefix.is_empty() {
                attr.local.start()
            } else {
                attr.prefix.start()
            };

            doc.attrs.push(Attribute {
                name: attr_name,
                value: mem::replace(&mut attr.value, String::new()),
                attr_pos,
                value_pos: attr.value_pos,
            });
        }
        attributes = pd.attrs_start_idx..doc.attrs.len();
        pd.attrs_start_idx = doc.attrs.len();
    }
    pd.tmp_attrs.clear();


    let tag_ns_uri = doc.namespaces.get_by_prefix(namespaces.clone(), tag_name.prefix.to_str());
    match end_token {
        xmlparser::ElementEnd::Empty => {
            doc.append(*parent_id,
                NodeKind::Element {
                    tag_name: ExpandedNameOwned {
                        ns: tag_ns_uri,
                        name: tag_name.name.to_str(),
                    },
                    attributes,
                    namespaces,
                },
                orig_pos_from_tag_name(&tag_name)
            );
        }
        xmlparser::ElementEnd::Close(prefix, local) => {
            let prefix_str = prefix.to_str();
            let local_str = local.to_str();

            if let NodeKind::Element { ref tag_name, .. } = doc.nodes[parent_id.0].kind {
                let parent_node = doc.get(*parent_id);
                let parent_prefix = parent_node.resolve_tag_name_prefix();

                if prefix_str != parent_prefix || local_str != tag_name.name {
                    return Err(Error::UnexpectedCloseTag {
                        expected: gen_qname_string(parent_prefix, tag_name.name),
                        actual: gen_qname_string(prefix_str, local_str),
                        pos: err_pos_from_tag_name(prefix, local, true),
                    });
                }
            }

            if let Some(id) = doc.nodes[parent_id.0].parent {
                *parent_id = id;
            } else {
                unreachable!("should be already checked by the xmlparser");
            }
        }
        xmlparser::ElementEnd::Open => {
            *parent_id = doc.append(*parent_id,
                NodeKind::Element {
                    tag_name: ExpandedNameOwned {
                        ns: tag_ns_uri,
                        name: tag_name.name.to_str(),
                    },
                    attributes,
                    namespaces,
                },
                orig_pos_from_tag_name(&tag_name)
            );
        }
    }

    Ok(())
}

fn process_text<'d>(
    text: StrSpan<'d>,
    parent_id: NodeId,
    nested: bool,
    pd: &mut ParserData<'d>,
    doc: &mut Document<'d>,
) -> Result<(), Error> {
    pd.u_buffer.clear();

    let mut s = Stream::from(text);
    while !s.at_end() {
        match parse_next_chunk(&mut s, &pd.entities)? {
            NextChunk::Byte(c) => {
                pd.u_buffer.push(c);
            }
            NextChunk::Char(c) => {
                let mut buf = [0xFF; 4];
                // `unwrap` is safe, `char` is 4 bytes long.
                write!(&mut buf[..], "{}", c).unwrap();
                for b in &buf {
                    if *b == 0xFF {
                        break;
                    }

                    pd.u_buffer.push(*b);
                }
            }
            NextChunk::Text(fragment) => {
                if nested {
                    let pos = s.gen_error_pos();
                    return Err(Error::NestedEntityReference(pos));
                }

                if !pd.u_buffer.is_empty() {
                    let s = trim_new_lines(&pd.u_buffer);
                    append_text(s, text.start(), parent_id, pd, doc);
                    pd.u_buffer.clear();
                }

                let mut parser = xmlparser::Tokenizer::from(fragment);
                parser.enable_fragment_mode();

                let mut tag_name = TagNameSpan::new(StrSpan::from(""), StrSpan::from(""));
                process_tokens(parser, true, parent_id, &mut tag_name, pd, doc)?;
                pd.u_buffer.clear();
            }
        }
    }

    if !pd.u_buffer.is_empty() {
        let s = trim_new_lines(&pd.u_buffer);
        append_text(s, text.start(), parent_id, pd, doc);
        pd.u_buffer.clear();
    }

    Ok(())
}

fn append_text(
    text: String,
    orig_pos: usize,
    parent_id: NodeId,
    pd: &mut ParserData,
    doc: &mut Document,
) {
    if let Some(xmlparser::Token::Cdata(_)) = pd.prev_node_type {
        if let Some(node) = doc.nodes.iter_mut().last() {
            if let NodeKind::Text(ref mut last_text) = node.kind {
                last_text.push_str(&text);
            }
        }
    } else {
        doc.append(parent_id, NodeKind::Text(text), orig_pos);
    }
}

// Translate \r\n and any \r that is not followed by \n to a single \n character.
//
// https://www.w3.org/TR/xml/#sec-line-ends
fn trim_new_lines(text: &[u8]) -> String {
    let mut text = text.to_vec();

    let mut i = 1;
    while i < text.len() {
        let prev_byte = text[i - 1];
        let curr_byte = text[i];

        if prev_byte == b'\r' && curr_byte == b'\n' {
            text.remove(i - 1);
        } else if prev_byte == b'\r' && curr_byte != b'\n' {
            text[i - 1] = b'\n';
        } else if curr_byte == b'\r' && i == text.len() - 1 {
            text[i] = b'\n';
        } else {
            i += 1;
        }
    }

    // `unwrap` is safe, because the input text was already a valid UTF-8 string.
    String::from_utf8(text).unwrap()
}

enum NextChunk<'a> {
    Byte(u8),
    Char(char),
    Text(StrSpan<'a>),
}

fn parse_next_chunk<'a>(
    s: &mut Stream<'a>,
    entities: &[(&str, StrSpan<'a>)],
) -> Result<NextChunk<'a>, Error> {
    debug_assert!(!s.at_end());

    // `unwrap` is safe, because we already checked that stream is not at end.
    // But we have an additional `debug_assert` above just in case.
    let c = s.curr_byte().unwrap();

    // Check for character/entity references.
    if c == b'&' {
        match s.try_consume_reference() {
            Some(Reference::CharRef(ch)) => {
                Ok(NextChunk::Char(ch))
            }
            Some(Reference::EntityRef(name)) => {
                match entities.iter().find(|v| v.0 == name) {
                    Some(v) => {
                        Ok(NextChunk::Text(v.1))
                    }
                    None => {
                        let pos = s.gen_error_pos();
                        Err(Error::UnknownEntityReference(name.into(), pos))
                    }
                }
            }
            None => {
                s.advance(1);
                Ok(NextChunk::Byte(c))
            }
        }
    } else {
        s.advance(1);
        Ok(NextChunk::Byte(c))
    }
}

fn process_cdata<'d>(
    cdata: StrSpan<'d>,
    parent_id: NodeId,
    pd: &mut ParserData,
    doc: &mut Document<'d>,
) {
    match pd.prev_node_type {
        Some(xmlparser::Token::Text(_)) | Some(xmlparser::Token::Whitespaces(_)) => {
            if let Some(node) = doc.nodes.iter_mut().last() {
                if let NodeKind::Text(ref mut text) = node.kind {
                    text.push_str(cdata.to_str());
                }
            }
        }
        _ => {
            doc.append(parent_id, NodeKind::Text(cdata.to_str().to_owned()), cdata.start());
        }
    }
}

// https://www.w3.org/TR/REC-xml/#AVNormalize
fn normalize_attribute(
    text: StrSpan,
    trim_spaces: bool,
    entities: &[(&str, StrSpan)],
    buffer: &mut Vec<u8>,
) -> Result<String, Error> {
    buffer.clear();
    _normalize_attribute(text, trim_spaces, entities, false, buffer)?;
    // `unwrap` is safe, because buffer must contain a valid UTF-8 string.
    Ok(str::from_utf8(buffer).unwrap().to_owned())
}

fn _normalize_attribute(
    text: StrSpan,
    trim_spaces: bool,
    entities: &[(&str, StrSpan)],
    nested: bool,
    buf: &mut Vec<u8>,
) -> Result<(), Error> {
    let mut s = Stream::from(text);
    while !s.at_end() {
        // `unwrap` is safe, because we already checked that stream is not at end.
        let c = s.curr_byte().unwrap();

        // Check for character/entity references.
        if c == b'&' {
            match s.try_consume_reference() {
                Some(Reference::CharRef(ch)) => {
                    let mut char_buf = [0xFF; 4];
                    // `unwrap` is safe, `char` is 4 bytes long.
                    write!(&mut char_buf[..], "{}", ch).unwrap();
                    for b in &char_buf {
                        if *b != 0xFF {
                            if nested {
                                push_byte(*b, None, buf);
                            } else {
                                buf.push(*b);
                            }
                        }
                    }

                    continue;
                }
                Some(Reference::EntityRef(name)) => {
                    if nested {
                        let pos = s.gen_error_pos();
                        return Err(Error::NestedEntityReference(pos));
                    }

                    match entities.iter().find(|v| v.0 == name) {
                        Some(v) => {
                            _normalize_attribute(v.1, trim_spaces, entities, true, buf)?;
                        }
                        None => {
                            let pos = s.gen_error_pos();
                            return Err(Error::UnknownEntityReference(name.into(), pos));
                        }
                    }

                    continue;
                }
                None => {
                    s.advance(1);
                }
            }
        } else {
            s.advance(1);
        }

        push_byte(c, s.get_curr_byte(), buf);
    }

    Ok(())
}

fn push_byte(mut c: u8, c2: Option<u8>, buf: &mut Vec<u8>) {
    // \r in \r\n should be ignored.
    if c == b'\r' && c2 == Some(b'\n') {
        return;
    }

    // \n, \r and \t should be converted into spaces.
    c = match c {
        b'\n' | b'\r' | b'\t' => b' ',
        _ => c,
    };

    buf.push(c);
}

fn gen_qname_string(prefix: &str, local: &str) -> String {
    if prefix.is_empty() {
        local.to_string()
    } else {
        format!("{}:{}", prefix, local)
    }
}

fn err_pos_from_span(text: StrSpan) -> TextPos {
    Stream::from(text).gen_error_pos()
}

fn err_pos_from_qname(prefix: StrSpan, local: StrSpan) -> TextPos {
    let err_span = if prefix.is_empty() { local } else { prefix };
    err_pos_from_span(err_span)
}

fn err_pos_from_tag_name(prefix: StrSpan, local: StrSpan, close_tag: bool) -> TextPos {
    let mut pos = err_pos_from_qname(prefix, local);

    if close_tag {
        pos.col -= 2; // jump before '</'
    } else {
        pos.col -= 1; // jump before '<'
    }

    pos
}

fn orig_pos_from_tag_name(tag_name: &TagNameSpan) -> usize {
    let span = if tag_name.prefix.is_empty() { tag_name.name } else { tag_name.prefix };
    span.start() - 1 // jump before '<'
}
