# roxmltree
[![Build Status](https://travis-ci.org/RazrFalcon/roxmltree.svg?branch=master)](https://travis-ci.org/RazrFalcon/roxmltree)
[![Crates.io](https://img.shields.io/crates/v/roxmltree.svg)](https://crates.io/crates/roxmltree)
[![Documentation](https://docs.rs/roxmltree/badge.svg)](https://docs.rs/roxmltree)
[![Rust 1.31+](https://img.shields.io/badge/rust-1.31+-orange.svg)](https://www.rust-lang.org)

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

For more details see [docs/parsing.md](https://github.com/RazrFalcon/roxmltree/blob/master/docs/parsing.md).

## Alternatives

| Feature/Crate                   | roxmltree        | [libxml2]           | [xmltree]        | [sxd-document]   | [minidom]        |
| ------------------------------- | :--------------: | :-----------------: | :--------------: | :--------------: | :--------------: |
| Element namespace resolving     | ✔                | ✔                   | ✔                | ~<sup>1</sup>    | ✔                |
| Attribute namespace resolving   | ✔                | ✔                   |                  | ✔                | ✔                |
| [Entity references]             | ✔                | ✔                   | ⚠                | ⚠                | ⚠                |
| [Character references]          | ✔                | ✔                   | ✔                | ✔                | ✔                |
| [Attribute-Value normalization] | ✔                | ✔                   |                  |                  |                  |
| Comments                        | ✔                | ✔                   |                  | ✔                | ✔                |
| Processing instructions         | ✔                | ✔                   | ✔                | ✔                |                  |
| UTF-8 BOM                       | ✔                | ✔                   | ⚠                | ⚠                | ✔                |
| Non UTF-8 input                 |                  | ✔                   |                  |                  |                  |
| Complete DTD support            |                  | ✔                   |                  |                  |                  |
| Position preserving<sup>2</sup> | ✔                | ✔                   |                  |                  |                  |
| HTML support                    |                  | ✔                   |                  |                  |                  |
| Tree modification               |                  | ✔                   | ✔                | ✔                | ✔                |
| Writing                         |                  | ✔                   | ✔                | ✔                | ✔                |
| No **unsafe**                   | ✔                |                     | ✔                |                  | ~<sup>3</sup>    |
| Language                        | Rust             | C                   | Rust             | Rust              | Rust            |
| Size overhead<sup>4</sup>       | ~67KiB           | ~1.4MiB<sup>4</sup> | ~118KiB          | ~138KiB           | **~63KiB**      |
| Dependencies                    | **1**            | ?<sup>5</sup>       | 2                | 2                 | 2               |
| Tested version                  | 0.9.1            | 2.9.8               | 0.10.0           | 0.3.2             | 0.11.1          |
| License                         | MIT / Apache-2.0 | MIT                 | MIT              | MIT               | MIT             |

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
3. In the `memchr` crate.
4. Binary size overhead according to [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat).
5. Depends on build flags.

There is also `elementtree` and `treexml` crates, but they are abandoned for a long time.

[Entity references]: https://www.w3.org/TR/REC-xml/#dt-entref
[Character references]: https://www.w3.org/TR/REC-xml/#NT-CharRef
[Attribute-Value Normalization]: https://www.w3.org/TR/REC-xml/#AVNormalize

[libxml2]: http://xmlsoft.org/
[xmltree]: https://crates.io/crates/xmltree
[sxd-document]: https://crates.io/crates/sxd-document
[minidom]: https://gitlab.com/xmpp-rs/xmpp-rs/-/tree/master/minidom-rs

## Performance

```text
test large_roxmltree     ... bench:   3,344,633 ns/iter (+/- 9,063)
test large_minidom       ... bench:   5,101,156 ns/iter (+/- 98,146)
test large_sdx_document  ... bench:   7,583,625 ns/iter (+/- 20,025)
test large_xmltree       ... bench:  20,792,783 ns/iter (+/- 523,851)

test medium_roxmltree    ... bench:     659,865 ns/iter (+/- 7,519)
test medium_minidom      ... bench:   1,176,302 ns/iter (+/- 7,317)
test medium_sdx_document ... bench:   2,510,734 ns/iter (+/- 18,054)
test medium_xmltree      ... bench:   7,678,284 ns/iter (+/- 174,265)

test tiny_roxmltree      ... bench:       4,178 ns/iter (+/- 23)
test tiny_minidom        ... bench:       7,468 ns/iter (+/- 88)
test tiny_sdx_document   ... bench:      18,202 ns/iter (+/- 91)
test tiny_xmltree        ... bench:      29,425 ns/iter (+/- 877)
```

*roxmltree* uses [xmlparser] internally,
while *sdx-document* uses its own implementation,
*xmltree* uses the [xml-rs]
and *minidom* uses [quick-xml].
Here is a comparison between *xmlparser*, *xml-rs* and *quick-xml*:

```text
test large_quick_xml     ... bench:   1,245,293 ns/iter (+/- 532,460)
test large_xmlparser     ... bench:   1,615,152 ns/iter (+/- 11,505)
test large_xmlrs         ... bench:  19,024,349 ns/iter (+/- 1,102,255)

test medium_quick_xml    ... bench:     246,507 ns/iter (+/- 3,300)
test medium_xmlparser    ... bench:     337,958 ns/iter (+/- 2,465)
test medium_xmlrs        ... bench:   6,944,242 ns/iter (+/- 29,862)

test tiny_quick_xml      ... bench:       2,328 ns/iter (+/- 67)
test tiny_xmlparser      ... bench:       2,578 ns/iter (+/- 931)
test tiny_xmlrs          ... bench:      27,343 ns/iter (+/- 3,299)
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
[quick-xml]: https://crates.io/crates/quick-xml
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
