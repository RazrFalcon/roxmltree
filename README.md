# roxmltree
![Build Status](https://github.com/RazrFalcon/roxmltree/workflows/Rust/badge.svg)
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
| Element namespace resolving     | ✓                | ✓                   | ✓                | ~<sup>1</sup>    | ✓                |
| Attribute namespace resolving   | ✓                | ✓                   |                  | ✓                | ✓                |
| [Entity references]             | ✓                | ✓                   | ×                | ×                | ×                |
| [Character references]          | ✓                | ✓                   | ✓                | ✓                | ✓                |
| [Attribute-Value normalization] | ✓                | ✓                   |                  |                  |                  |
| Comments                        | ✓                | ✓                   |                  | ✓                |                  |
| Processing instructions         | ✓                | ✓                   | ✓                | ✓                |                  |
| UTF-8 BOM                       | ✓                | ✓                   | ×                | ×                | ✓                |
| Non UTF-8 input                 |                  | ✓                   |                  |                  |                  |
| Complete DTD support            |                  | ✓                   |                  |                  |                  |
| Position preserving<sup>2</sup> | ✓                | ✓                   |                  |                  |                  |
| HTML support                    |                  | ✓                   |                  |                  |                  |
| Tree modification               |                  | ✓                   | ✓                | ✓                | ✓                |
| Writing                         |                  | ✓                   | ✓                | ✓                | ✓                |
| No **unsafe**                   | ✓                |                     | ✓                |                  | ~<sup>3</sup>    |
| Language                        | Rust             | C                   | Rust             | Rust             | Rust             |
| Size overhead<sup>4</sup>       | ~61KiB           | ~1.4MiB<sup>5</sup> | ~106KiB          | ~121KiB          | **~58KiB**       |
| Dependencies                    | **1**            | ?<sup>5</sup>       | 2                | 2                | 2                |
| Tested version                  | 0.14.0           | 2.9.8               | 0.10.2           | 0.3.2            | 0.12.0           |
| License                         | MIT / Apache-2.0 | MIT                 | MIT              | MIT              | MIT              |

Legend:

- ✓ - supported
- × - parsing error
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

### Parsing

```text
test large_roxmltree     ... bench:   3,123,941 ns/iter (+/- 19,992)
test large_minidom       ... bench:   4,969,218 ns/iter (+/- 163,727)
test large_sdx_document  ... bench:   7,266,856 ns/iter (+/- 26,998)
test large_xmltree       ... bench:  21,354,608 ns/iter (+/- 136,311)

test medium_roxmltree    ... bench:     547,522 ns/iter (+/- 5,956)
test medium_minidom      ... bench:   1,223,620 ns/iter (+/- 16,180)
test medium_sdx_document ... bench:   2,470,063 ns/iter (+/- 24,159)
test medium_xmltree      ... bench:   8,083,860 ns/iter (+/- 25,363)

test tiny_roxmltree      ... bench:       4,170 ns/iter (+/- 41)
test tiny_minidom        ... bench:       7,495 ns/iter (+/- 81)
test tiny_sdx_document   ... bench:      17,411 ns/iter (+/- 203)
test tiny_xmltree        ... bench:      29,522 ns/iter (+/- 223)
```

*roxmltree* uses [xmlparser] internally,
while *sdx-document* uses its own implementation,
*xmltree* uses the [xml-rs]
and *minidom* uses [quick-xml].
Here is a comparison between *xmlparser*, *xml-rs* and *quick-xml*:

```text
test large_quick_xml     ... bench:   1,286,273 ns/iter (+/- 27,174)
test large_xmlparser     ... bench:   1,742,202 ns/iter (+/- 11,616)
test large_xmlrs         ... bench:  19,615,797 ns/iter (+/- 105,848)

test medium_quick_xml    ... bench:     248,169 ns/iter (+/- 3,885)
test medium_xmlparser    ... bench:     386,658 ns/iter (+/- 1,721)
test medium_xmlrs        ... bench:   7,387,753 ns/iter (+/- 18,668)

test tiny_quick_xml      ... bench:       2,382 ns/iter (+/- 29)
test tiny_xmlparser      ... bench:       2,788 ns/iter (+/- 20)
test tiny_xmlrs          ... bench:      27,619 ns/iter (+/- 262)
```

### Iteration

```text
test xmltree_iter_descendants_expensive     ... bench:     436,684 ns/iter (+/- 7,851)
test roxmltree_iter_descendants_expensive   ... bench:     470,459 ns/iter (+/- 6,233)
test minidom_iter_descendants_expensive     ... bench:     785,847 ns/iter (+/- 51,495)

test roxmltree_iter_descendants_inexpensive ... bench:      36,759 ns/iter (+/- 684)
test xmltree_iter_descendants_inexpensive   ... bench:     168,541 ns/iter (+/- 1,885)
test minidom_iter_descendants_inexpensive   ... bench:     215,615 ns/iter (+/- 38,101)
```

Where expensive refers to the matching done on each element. In these
benchmarks, *expensive* means searching for any node in the document which
contains a string. And *inexpensive* means searching for any element with a
particular name.

### Notes

You can try running the benchmarks yourself by running `cargo bench` in the `benches` dir.

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
