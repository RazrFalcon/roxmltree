# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Changed
- `Attribute::namespace` will return `Option` now.
- `ExpandedName::namespace` will return `Option` now.
- `Namespace::name` will return `Option` now.
- `Node::resolve_tag_name_prefix` will return `Option` now.
- `Node::lookup_namespace_uri` accepts `Option<&str>` and not `&str` now.
- Performance optimizations.

### Removed
- `ExpandedName::has_namespace`. `ExpandedName::namespace` will return `Option` now.

[Unreleased]: https://github.com/RazrFalcon/roxmltree/compare/v0.1.0...HEAD
