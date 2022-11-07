# roxmltree
![Build Status](https://github.com/RazrFalcon/roxmltree/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/roxmltree.svg)](https://crates.io/crates/roxmltree)
[![Documentation](https://docs.rs/roxmltree/badge.svg)](https://docs.rs/roxmltree)
[![Rust 1.36+](https://img.shields.io/badge/rust-1.36+-orange.svg)](https://www.rust-lang.org)

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
[minidom]: https://gitlab.com/xmpp-rs/xmpp-rs/-/tree/main/minidom

## Performance

### Parsing

```text
test large_roxmltree     ... bench:   1,553,308 ns/iter (+/- 58,306)
test large_libxml        ... bench:   1,933,786 ns/iter (+/- 58,435)
test large_sdx_document  ... bench:   3,469,016 ns/iter (+/- 227,326)
test large_minidom       ... bench:   3,532,325 ns/iter (+/- 108,046)
test large_xmltree       ... bench:  12,009,212 ns/iter (+/- 749,100)

test medium_roxmltree    ... bench:     419,176 ns/iter (+/- 18,119)
test medium_libxml       ... bench:     692,290 ns/iter (+/- 155,278)
test medium_minidom      ... bench:     781,737 ns/iter (+/- 33,125)
test medium_sdx_document ... bench:   1,237,014 ns/iter (+/- 39,868)
test medium_xmltree      ... bench:   3,898,475 ns/iter (+/- 213,596)

test tiny_roxmltree      ... bench:       2,165 ns/iter (+/- 25)
test tiny_minidom        ... bench:       5,645 ns/iter (+/- 82)
test tiny_libxml         ... bench:       6,446 ns/iter (+/- 163)
test tiny_sdx_document   ... bench:       8,678 ns/iter (+/- 739)
test tiny_xmltree        ... bench:      15,711 ns/iter (+/- 278)
```

*roxmltree* uses [xmlparser] internally,
while *sdx-document* uses its own implementation,
*xmltree* uses the [xml-rs]
and *minidom* uses [quick-xml].
Here is a comparison between *xmlparser*, *xml-rs* and *quick-xml*:

```text
test large_xmlparser     ... bench:     732,620 ns/iter (+/- 6,615)
test large_quick_xml     ... bench:     953,197 ns/iter (+/- 42,547)
test large_xmlrs         ... bench:   9,932,179 ns/iter (+/- 271,766)

test medium_quick_xml    ... bench:     222,990 ns/iter (+/- 21,897)
test medium_xmlparser    ... bench:     252,363 ns/iter (+/- 2,298)
test medium_xmlrs        ... bench:   3,617,399 ns/iter (+/- 91,085)

test tiny_xmlparser      ... bench:       1,040 ns/iter (+/- 7)
test tiny_quick_xml      ... bench:       1,634 ns/iter (+/- 25)
test tiny_xmlrs          ... bench:      14,095 ns/iter (+/- 217)
```

### Iteration

```text
test xmltree_iter_descendants_expensive     ... bench:     343,790 ns/iter (+/- 2,767)
test roxmltree_iter_descendants_expensive   ... bench:     562,800 ns/iter (+/- 7,487)
test minidom_iter_descendants_expensive     ... bench:     662,709 ns/iter (+/- 2,380)

test roxmltree_iter_descendants_inexpensive ... bench:      25,648 ns/iter (+/- 949)
test xmltree_iter_descendants_inexpensive   ... bench:      76,779 ns/iter (+/- 1,026)
test minidom_iter_descendants_inexpensive   ... bench:     114,691 ns/iter (+/- 1,803)
```

Where expensive refers to the matching done on each element. In these
benchmarks, *expensive* means searching for any node in the document which
contains a string. And *inexpensive* means searching for any element with a
particular name.

### Notes

The benchmarks were taken on a 2021 MacBook Pro with the Apple M1 Pro chip with 16GB of RAM.
You can try running the benchmarks yourself by running `cargo bench` in the `benches` dir.

- Since all libraries have a different XML support, benchmarking is a bit pointless.
- Tree crates may use different *xml-rs* crate versions.
- We bench *libxml2* using the *[rust-libxml]* wrapper crate
- *quick-xml* is faster than *xmlparser* because it's more forgiving for the input,
  while *xmlparser* is very strict and does a lot of checks, which are expensive.
  So performance difference is mainly due to validation.

[xml-rs]: https://crates.io/crates/xml-rs
[quick-xml]: https://crates.io/crates/quick-xml
[xmlparser]: https://crates.io/crates/xmlparser
[rust-libxml]: https://github.com/KWARC/rust-libxml

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
