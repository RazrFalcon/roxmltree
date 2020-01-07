# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
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

[Unreleased]: https://github.com/RazrFalcon/roxmltree/compare/v0.8.0...HEAD
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
