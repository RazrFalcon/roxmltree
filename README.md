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
| Tested version                  | 0.16.0           | 2.9.8               | 0.10.2           | 0.3.2            | 0.12.0           |
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
test large_roxmltree     ... bench:   1,550,600 ns/iter (+/- 90,292)
test large_libxml        ... bench:   3,257,226 ns/iter (+/- 203,441)
test large_sdx_document  ... bench:   3,809,712 ns/iter (+/- 95,477)
test large_minidom       ... bench:   5,393,247 ns/iter (+/- 152,617)
test large_xmltree       ... bench:  16,149,937 ns/iter (+/- 464,889)

test medium_roxmltree    ... bench:     431,244 ns/iter (+/- 33,706)
test medium_libxml       ... bench:     937,626 ns/iter (+/- 25,933)
test medium_minidom      ... bench:   1,258,619 ns/iter (+/- 15,863)
test medium_sdx_document ... bench:   1,497,293 ns/iter (+/- 44,276)
test medium_xmltree      ... bench:   4,962,228 ns/iter (+/- 192,942)

test tiny_roxmltree      ... bench:       2,378 ns/iter (+/- 140)
test tiny_minidom        ... bench:       9,188 ns/iter (+/- 1,641)
test tiny_libxml         ... bench:       9,238 ns/iter (+/- 240)
test tiny_sdx_document   ... bench:      10,578 ns/iter (+/- 264)
test tiny_xmltree        ... bench:      22,561 ns/iter (+/- 603)
```

*roxmltree* uses [xmlparser] internally,
while *sdx-document* uses its own implementation,
*xmltree* uses the [xml-rs]
and *minidom* uses [quick-xml].
Here is a comparison between *xmlparser*, *xml-rs* and *quick-xml*:

```text
test large_xmlparser     ... bench:     760,960 ns/iter (+/- 15,320)
test large_quick_xml     ... bench:     977,787 ns/iter (+/- 32,916)
test large_xmlrs         ... bench:  13,112,612 ns/iter (+/- 311,702)

test medium_quick_xml    ... bench:     229,251 ns/iter (+/- 10,529)
test medium_xmlparser    ... bench:     263,752 ns/iter (+/- 12,571)
test medium_xmlrs        ... bench:   4,671,499 ns/iter (+/- 97,025)

test tiny_quick_xml      ... bench:       1,806 ns/iter (+/- 56)
test tiny_xmlparser      ... bench:       1,070 ns/iter (+/- 15)
test tiny_xmlrs          ... bench:      19,276 ns/iter (+/- 579)
```

### Iteration

```text
test xmltree_iter_descendants_expensive     ... bench:     406,101 ns/iter (+/- 10,184)
test roxmltree_iter_descendants_expensive   ... bench:     579,130 ns/iter (+/- 6,607)
test minidom_iter_descendants_expensive     ... bench:     717,989 ns/iter (+/- 31,789)

test roxmltree_iter_descendants_inexpensive ... bench:      22,903 ns/iter (+/- 780)
test xmltree_iter_descendants_inexpensive   ... bench:     127,131 ns/iter (+/- 3,250)
test minidom_iter_descendants_inexpensive   ... bench:     143,045 ns/iter (+/- 16,683)

test roxmltree_iter_children                ... bench:       1,398 ns/iter (+/- 68)
```

Where expensive refers to the matching done on each element. In these
benchmarks, *expensive* means searching for any node in the document which
contains a string. And *inexpensive* means searching for any element with a
particular name.

### Notes

The benchmarks were taken on a Apple M1.
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

## Memory Overhead

`roxmltree` tries to use as little memory as possible to allow parsing
very large (multi-GB) XML files.

The peak memory usage doesn't directly correlates with the file size
but rather with the amount of nodes and attributes a file has.
How many attributes had to be normalized (i.e. allocated).
And how many text nodes had to be preprocessed (i.e. allocated).

`roxmltree` never allocates element and attribute names, processing instructions
and comments.

By disabling the `positions` feature, you can shave by 8 bytes from each node and attribute.

On average, the overhead is around 6-8x the file size.
For example, our 1.1GB sample XML will peak at 7.6GB RAM with default features enabled
and at 6.8GB RAM when `positions` is disabled.

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
