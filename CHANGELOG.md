# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [0.14.1] - 2021-04-01
### Changed
- The `std` feature is enabled by default now.

## [0.14.0] - 2020-12-27
### Added
- An ability to reject XML with DTD in it.
- The library is no_std + alloc now.
- `ParsingOptions`
- `Document::parse_with_options`
- `Error::DtdDetected`

## [0.13.1] - 2020-12-19
### Added
- `Debug` for all public types.

## [0.13.0] - 2020-06-20
### Changed
- Better `ExpandedName` lifetimes. Thanks to [@eduardosm](https://github.com/eduardosm).

## [0.12.0] - 2020-06-20
### Changed
- Re-release 0.11.1, since it had a breaking change.

## [0.11.1] - 2020-06-19
### Changed
- Extend `ExpandedName` lifetime. Thanks to [@rkusa](https://github.com/rkusa).

## [0.11.0] - 2020-04-19
### Added
- Implement `Ord`, `PartialOrd` and `Hash` for `Node`. Thanks to [@tomjw64].
- `NodeId`, `Document::get_node` and `Node::id`. Thanks to [@tomjw64].

### Changed
- The input data size is limited by 4GiB now.
- `Node` can be accessed from multiple threads now.
- Reduce `Node` memory usage.
- Greatly optimized `Descendants` iterator. Up to 5x faster in some cases. Thanks to [@tomjw64].
- Heavily reduce memory usage when document has a lot of namespaces. Thanks to [@tomjw64].

### Removed
- `Node::traverse`, `Traverse` and `Edge`. Use `Node::descendants` instead.

## [0.10.1] - 2020-03-28
### Fixed
- `Node::prev_sibling_element` and `Node::next_sibling_element`
  were returning the current node.

## [0.10.0] - 2020-03-18
### Added
- `Document::input_text`

### Changed
- `Ancestors`, `PrevSiblings`, `NextSiblings`, `FirstChildren` and `LastChildren`
  were replaced with `AxisIter`.

### Fixed
- Root node range.

## [0.9.1] - 2020-02-09
### Changed
- A better entity loop detection. A document can have an unlimited
  number of references at zero depth now.

## [0.9.0] - 2020-01-07
### Changed
- Moved to Rust 2018.
- `xmlparser` updated.

## [0.8.0] - 2019-12-21
### Added
- `Error::MalformedEntityReference`

### Changed
- Malformed entity reference is an error now.
- Escaped `<` in attribute inside an ENTITY is an error now.
- `xmlparser` updated with multiple fixes.

## [0.7.3] - 2019-11-14
### Changed
- Use unconstrained lifetimes for the `attribute` functions.
  By [myrrlyn](https://github.com/myrrlyn).

## [0.7.2] - 2019-11-07
### Changed
- Use longer lifetimes in `Document::root_element`.
  By [myrrlyn](https://github.com/myrrlyn).

## [0.7.1] - 2019-09-14
### Changed
- Update `xmlparser`.

## [0.7.0] - 2019-08-06
### Added
- `Node::prev_sibling_element` and `Node::next_sibling_element`.

### Changed
- **(breaking)** `Node::ancestors` includes the current node now.
- `Attribute` is cloneable now.

### Fixed
- Namespaces resolving with equal URI's.

### Removed
- `Node::resolve_tag_name_prefix`.

## [0.6.1] - 2019-06-18
### Fixed
- Namespace resolving.

## [0.6.0] - 2019-03-03
### Added
- `Error::UnknownNamespace`.

### Fixed
- Unknown namespace prefixes will cause an error now.

## [0.5.0] - 2019-02-27
### Added
- `Node::range`.
- `Node::attribute_node`.
- `Attribute::range`.
- `Attribute::value_range`.

### Changed
- Rename `text_pos_from` into `text_pos_at`.

### Removed
- `Node::pos`. Use `Node::range` instead.
- `Node::node_pos`. Use `doc.text_pos_at(node.range().start)` instead.
- `Node::attribute_pos`.
- `Node::attribute_value_pos`.
- `Attribute::pos`. Use `Attribute::range` instead.
- `Attribute::value_pos`. Use `Attribute::value_range` instead.

## [0.4.1] - 2019-01-02
### Changed
- Use longer lifetimes in return types. By [tmiasko](https://github.com/tmiasko).

## [0.4.0] - 2018-12-13
### Added
- `Error::pos()`.

## [0.3.0] - 2018-10-29
### Changed
- Store text nodes as `&str` when possible. On an XML with a lot of simple text can be ~2x faster.
- `Document` no longer implements `PartialEq`.

### Fixed
- Entity and character references resolving inside a text.

## [0.2.0] - 2018-10-08
### Added
- `Error::EntityReferenceLoop`.
- Nested entity references support.

### Changed
- `Attribute::namespace` will return `Option` now.
- `ExpandedName::namespace` will return `Option` now.
- `Namespace::name` will return `Option` now.
- `Node::resolve_tag_name_prefix` will return `Option` now.
- `Node::lookup_namespace_uri` accepts `Option<&str>` and not `&str` now.
- Performance optimizations.

### Removed
- `ExpandedName::has_namespace`. `ExpandedName::namespace` will return `Option` now.
- `Error::NestedEntityReference`.

[@tomjw64]: https://github.com/tomjw64

[Unreleased]: https://github.com/RazrFalcon/roxmltree/compare/v0.14.1..HEAD
[0.14.1]: https://github.com/RazrFalcon/roxmltree/compare/v0.14.0...v0.14.1
[0.14.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.13.1...v0.14.0
[0.13.1]: https://github.com/RazrFalcon/roxmltree/compare/v0.13.0...v0.13.1
[0.13.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.11.1...v0.12.0
[0.11.1]: https://github.com/RazrFalcon/roxmltree/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.10.1...v0.11.0
[0.10.1]: https://github.com/RazrFalcon/roxmltree/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.9.1...v0.10.0
[0.9.1]: https://github.com/RazrFalcon/roxmltree/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.7.3...v0.8.0
[0.7.3]: https://github.com/RazrFalcon/roxmltree/compare/v0.7.2...v0.7.3
[0.7.2]: https://github.com/RazrFalcon/roxmltree/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/RazrFalcon/roxmltree/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/RazrFalcon/roxmltree/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/RazrFalcon/roxmltree/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/RazrFalcon/roxmltree/compare/v0.1.0...v0.2.0
