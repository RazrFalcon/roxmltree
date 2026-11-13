use core::ops::Range;

use crate::{Attributes, ExpandedName, NamespaceIter, NodeId, PI};

/// A semantically checked parser event. Borrowed values are valid during the callback.
/// A later input error can still cause the complete parse to fail.
#[derive(Debug)]
pub enum ValidatedEvent<'a, 'input: 'a> {
    /// An opening element, after namespace and attribute validation.
    ElementStart {
        /// The node's index in the equivalent indexed document.
        id: NodeId,
        /// The parent index; zero denotes the implicit document root.
        parent: NodeId,
        /// The resolved element name.
        tag_name: ExpandedName<'a, 'input>,
        /// Normalised attributes in their original order.
        attributes: Attributes<'a, 'input>,
        /// Effective namespace bindings, including inherited bindings.
        namespaces: NamespaceIter<'a, 'input>,
        /// The byte offset of the opening `<` in the source.
        start: usize,
    },
    /// A closing element, including a self-closing element.
    ElementEnd {
        /// The index from the matching opening event.
        id: NodeId,
        /// The byte offset immediately after the closing `>`.
        end: usize,
    },
    /// One logical text node, with adjacent text, CDATA and entity text merged.
    Text {
        /// The node's index in the equivalent indexed document.
        id: NodeId,
        /// The parent index.
        parent: NodeId,
        /// Normalised text, as returned by the indexed node.
        text: &'a str,
        /// The source range of the first merged fragment, as in the indexed node.
        range: Range<usize>,
    },
    /// A comment, including comments in the DTD.
    Comment {
        /// The node's index in the equivalent indexed document.
        id: NodeId,
        /// The parent index.
        parent: NodeId,
        /// The comment content.
        text: &'a str,
        /// The comment's source range.
        range: Range<usize>,
    },
    /// A processing instruction, including instructions in the DTD.
    ProcessingInstruction {
        /// The node's index in the equivalent indexed document.
        id: NodeId,
        /// The parent index.
        parent: NodeId,
        /// The instruction's target and value.
        pi: PI<'input>,
        /// The instruction's source range.
        range: Range<usize>,
    },
}
