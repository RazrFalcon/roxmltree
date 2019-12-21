# roxmltree
[![Build Status](https://travis-ci.org/RazrFalcon/roxmltree.svg?branch=master)](https://travis-ci.org/RazrFalcon/roxmltree)
[![Crates.io](https://img.shields.io/crates/v/roxmltree.svg)](https://crates.io/crates/roxmltree)
[![Documentation](https://docs.rs/roxmltree/badge.svg)](https://docs.rs/roxmltree)
[![Rust 1.18+](https://img.shields.io/badge/rust-1.18+-orange.svg)](https://www.rust-lang.org)

Represents an [XML 1.0](https://www.w3.org/TR/xml/) document as a read-only tree.

```rust
// Find element by id.
let doc = roxmltree::Document::parse("<rect id='rect1'/>").unwrap();
let elem = doc.descendants().find(|n| n.attribute("id") == Some("rect1")).unwrap();
assert!(elem.has_tag_name("rect"));
```

## Why read-only?

Because in some cases all you need is to retrieve some data from an XML document.
And for such cases, we can make a lot of optimizations.

As for *roxmltree*, it's fast not only because it's read-only, but also because
it uses [xmlparser], which is many times faster than [xml-rs].
See the [Performance](#performance) section for details.

## Parsing behavior

Sadly, XML can be parsed in many different ways. *roxmltree* tries to mimic the
behavior of Python's [lxml](https://lxml.de/).
But unlike *lxml*, *roxmltree* does support comments outside the root element.

Fo more details see [docs/parsing.md](https://github.com/RazrFalcon/roxmltree/blob/master/docs/parsing.md).

## Alternatives

| Feature/Crate                   | roxmltree        | [libxml2]           | [xmltree]        | [elementtree]    | [sxd-document]   | [treexml]        |
| ------------------------------- | :--------------: | :-----------------: | :--------------: | :--------------: | :--------------: | :--------------: |
| Element namespace resolving     | ✔                | ✔                   | ✔                | ✔               | ~<sup>1</sup>     |                  |
| Attribute namespace resolving   | ✔                | ✔                   |                  |                  | ✔                |                  |
| [Entity references]             | ✔                | ✔                   | ⚠                | ⚠                | ⚠             | ⚠                |
| [Character references]          | ✔                | ✔                   | ✔                | ✔                | ✔                | ✔                |
| [Attribute-Value normalization] | ✔                | ✔                   |                  |                  |                  |                  |
| Comments                        | ✔                | ✔                   |                  |                  | ✔                |                  |
| Processing instructions         | ✔                | ✔                   | ⚠                |                  | ✔               |                  |
| UTF-8 BOM                       | ✔                | ✔                   | ⚠               | ⚠               | ⚠               | ⚠                |
| Non UTF-8 input                 |                  | ✔                    |                  |                  |                  |                  |
| Complete DTD support            |                  | ✔                   |                  |                  |                  |                  |
| Position preserving<sup>2</sup> | ✔                | ✔                   |                 |                 |                 |                  |
| HTML support                    |                  | ✔                   |                  |                  |                  |                  |
| Tree modification               |                  | ✔                   | ✔                | ✔                | ✔                | ✔                |
| Writing                         |                  | ✔                   | ✔                | ✔                | ✔                | ✔                |
| No **unsafe**                   | ✔                |                     | ✔                | ~<sup>3</sup>    |                  | ✔                |
| Language                        | Rust             | C                   | Rust             | Rust             | Rust             | Rust             |
| Size overhead<sup>4</sup>       | **~73KiB**       | ~1.4MiB<sup>5</sup> | ~80KiB           | ~96KiB           | ~135KiB          | ~110KiB          |
| Dependencies                    | **1**            | ?<sup>5</sup>       | 2                | 18               | 2                | 14               |
| Tested version                  | 0.8.0            | 2.9.8               | 0.9.0            | 0.5.0            | 0.3.0            | 0.7.0            |
| License                         | MIT / Apache-2.0 | MIT                 | MIT              | BSD-3-Clause     | MIT              | MIT              |

Legend:

- ✔ - supported
- ⚠ - parsing error
- ~ - partial
- *nothing* - not supported

Notes:

1. No default namespace propagation.
2. *roxmltree* keeps all node and attribute positions in the original document,
   so you can easily retrieve it if you need it.
   See [examples/print_pos.rs](examples/print_pos.rs) for details.
3. In the `string_cache` crate.
4. Binary size overhead according to [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat).
5. Depends on build flags.

[Entity references]: https://www.w3.org/TR/REC-xml/#dt-entref
[Character references]: https://www.w3.org/TR/REC-xml/#NT-CharRef
[Attribute-Value Normalization]: https://www.w3.org/TR/REC-xml/#AVNormalize

[libxml2]: http://xmlsoft.org/
[xmltree]: https://crates.io/crates/xmltree
[elementtree]: https://crates.io/crates/elementtree
[treexml]: https://crates.io/crates/treexml
[sxd-document]: https://crates.io/crates/sxd-document

## Performance

```text
test large_roxmltree     ... bench:   3,976,162 ns/iter (+/- 16,229)
test large_sdx_document  ... bench:   7,501,511 ns/iter (+/- 33,603)
test large_xmltree       ... bench:  20,821,266 ns/iter (+/- 80,124)
test large_elementtree   ... bench:  21,388,702 ns/iter (+/- 115,590)
test large_treexml       ... bench:  21,469,671 ns/iter (+/- 192,099)

test medium_roxmltree    ... bench:     732,136 ns/iter (+/- 6,410)
test medium_sdx_document ... bench:   2,548,236 ns/iter (+/- 14,502)
test medium_elementtree  ... bench:   8,505,173 ns/iter (+/- 26,123)
test medium_treexml      ... bench:   8,146,522 ns/iter (+/- 19,378)
test medium_xmltree      ... bench:   8,217,647 ns/iter (+/- 22,061)

test tiny_roxmltree      ... bench:       5,039 ns/iter (+/- 46)
test tiny_sdx_document   ... bench:      18,204 ns/iter (+/- 145)
test tiny_elementtree    ... bench:      30,865 ns/iter (+/- 280)
test tiny_treexml        ... bench:      30,698 ns/iter (+/- 468)
test tiny_xmltree        ... bench:      30,338 ns/iter (+/- 231)
```

*roxmltree* uses [xmlparser] internally,
while *sdx-document* uses its own implementation and *xmltree*, *elementtree*
and *treexml* use the [xml-rs] crate.
Here is a comparison between *xmlparser*, *xml-rs* and *quick-xml*:

```text
test large_quick_xml     ... bench:   1,220,067 ns/iter (+/- 20,723)
test large_xmlparser     ... bench:   2,079,871 ns/iter (+/- 12,220)
test large_xmlrs         ... bench:  19,628,313 ns/iter (+/- 241,729)

test medium_quick_xml    ... bench:     246,421 ns/iter (+/- 17,438)
test medium_xmlparser    ... bench:     408,831 ns/iter (+/- 4,351)
test medium_xmlrs        ... bench:   7,430,009 ns/iter (+/- 40,350)

test tiny_quick_xml      ... bench:       2,329 ns/iter (+/- 67)
test tiny_xmlparser      ... bench:       3,313 ns/iter (+/- 22)
test tiny_xmlrs          ... bench:      28,511 ns/iter (+/- 232)
```

You can try it yourself by running `cargo bench` in the `benches` dir.

Notes:

- Since all libraries have a different XML support, benchmarking is a bit pointless.
- Tree crates may use different *xml-rs* crate versions.
- We do not bench the libxml2, because `xmlReadFile()` will parse only an XML structure,
  without attributes normalization and stuff. So it's hard to compare.
  And we have to use a separate benchmark utility.
- *quick-xml* is faster than *xmlparser* because it's more forgiving for the input,
  while *xmlparser* is very strict and does a lot of checks, which are expensive.
  So performance difference is mainly due to validation.

[xml-rs]: https://crates.io/crates/xml-rs
[xmlparser]: https://crates.io/crates/xmlparser

## Safety

- This library must not panic. Any panic should be considered a critical bug and reported.
- This library forbids `unsafe` code.

## Non-goals

- Complete XML support.
- Tree modification and writing.
- XPath/XQuery.

## API

This library uses Rust's idiomatic API based on iterators.
In case you are more familiar with browser/JS DOM APIs - you can check out
[tests/dom-api.rs](tests/dom-api.rs) to see how it can be converted into a Rust one.

## License

Licensed under either of

- [Apache License v2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
