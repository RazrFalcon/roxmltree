# roxmltree
![Build Status](https://github.com/RazrFalcon/roxmltree/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/roxmltree.svg)](https://crates.io/crates/roxmltree)
[![Documentation](https://docs.rs/roxmltree/badge.svg)](https://docs.rs/roxmltree)
[![Rust 1.36+](https://img.shields.io/badge/rust-1.36+-orange.svg)](https://www.rust-lang.org)

Represents an [XML 1.0](https://www.w3.org/TR/xml/) document as a read-only tree.

```rust
// Find element by id.
let doc = roxmltree::Document::parse("<rect id='rect1'/>")?;
let elem = doc.descendants().find(|n| n.attribute("id") == Some("rect1"))?;
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

| Feature/Crate                   | roxmltree        | [libxml2]           | [xmltree]        | [sxd-document]   |
| ------------------------------- | :--------------: | :-----------------: | :--------------: | :--------------: |
| Element namespace resolving     | ✓                | ✓                   | ✓                | ~<sup>1</sup>    |
| Attribute namespace resolving   | ✓                | ✓                   |                  | ✓                |
| [Entity references]             | ✓                | ✓                   | ×                | ×                |
| [Character references]          | ✓                | ✓                   | ✓                | ✓                |
| [Attribute-Value normalization] | ✓                | ✓                   |                  |                  |
| Comments                        | ✓                | ✓                   |                  | ✓                |
| Processing instructions         | ✓                | ✓                   | ✓                | ✓                |
| UTF-8 BOM                       | ✓                | ✓                   | ×                | ×                |
| Non UTF-8 input                 |                  | ✓                   |                  |                  |
| Complete DTD support            |                  | ✓                   |                  |                  |
| Position preserving<sup>2</sup> | ✓                | ✓                   |                  |                  |
| HTML support                    |                  | ✓                   |                  |                  |
| Tree modification               |                  | ✓                   | ✓                | ✓                |
| Writing                         |                  | ✓                   | ✓                | ✓                |
| No **unsafe**                   | ✓                |                     | ✓                |                  |
| Language                        | Rust             | C                   | Rust             | Rust             |
| Size overhead<sup>4</sup>       | **~55KiB**       | ~1.4MiB<sup>5</sup> | ~78KiB           | ~102KiB          |
| Dependencies                    | **1**            | ?<sup>5</sup>       | 2                | 2                |
| Tested version                  | 0.18.0           | Apple-provided      | 0.10.3           | 0.3.2            |
| License                         | MIT / Apache-2.0 | MIT                 | MIT              | MIT              |

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

## Performance

### Parsing

```text
test huge_roxmltree      ... bench:   3,147,424 ns/iter (+/- 49,153)
test huge_libxml         ... bench:   6,850,666 ns/iter (+/- 306,180)
test huge_sdx_document   ... bench:   9,440,412 ns/iter (+/- 117,106)
test huge_xmltree        ... bench:  41,662,316 ns/iter (+/- 850,360)

test large_roxmltree     ... bench:   1,594,201 ns/iter (+/- 27,425)
test large_libxml        ... bench:   3,250,606 ns/iter (+/- 140,201)
test large_sdx_document  ... bench:   4,242,162 ns/iter (+/- 99,740)
test large_xmltree       ... bench:  13,980,228 ns/iter (+/- 229,363)

test medium_roxmltree    ... bench:     418,929 ns/iter (+/- 4,843)
test medium_libxml       ... bench:     950,984 ns/iter (+/- 34,099)
test medium_sdx_document ... bench:   1,618,270 ns/iter (+/- 23,466)
test medium_xmltree      ... bench:   4,315,974 ns/iter (+/- 31,849)

test tiny_roxmltree      ... bench:       2,654 ns/iter (+/- 103)
test tiny_libxml         ... bench:       8,931 ns/iter (+/- 235)
test tiny_sdx_document   ... bench:      11,658 ns/iter (+/- 82)
test tiny_xmltree        ... bench:      20,215 ns/iter (+/- 303)
```

*roxmltree* uses [xmlparser] internally,
while *sdx-document* uses its own implementation,
*xmltree* uses the [xml-rs].
Here is a comparison between *xmlparser*, *xml-rs* and *quick-xml*:

```text
test huge_xmlparser      ... bench:   1,672,879 ns/iter (+/- 20,140)
test huge_quick_xml      ... bench:   2,396,037 ns/iter (+/- 39,752)
test huge_xmlrs          ... bench:  36,258,312 ns/iter (+/- 180,438)

test large_xmlparser     ... bench:     730,787 ns/iter (+/- 22,924)
test large_quick_xml     ... bench:   1,250,053 ns/iter (+/- 21,943)
test large_xmlrs         ... bench:  11,239,516 ns/iter (+/- 76,937)

test medium_quick_xml    ... bench:     206,232 ns/iter (+/- 2,157)
test medium_xmlparser    ... bench:     240,737 ns/iter (+/- 4,531)
test medium_xmlrs        ... bench:   3,975,916 ns/iter (+/- 44,967)

test tiny_xmlparser      ... bench:       1,078 ns/iter (+/- 17)
test tiny_quick_xml      ... bench:       2,233 ns/iter (+/- 70)
test tiny_xmlrs          ... bench:      17,155 ns/iter (+/- 429)
```

### Iteration

```text
test roxmltree_iter_descendants_expensive   ... bench:     255,882 ns/iter (+/- 1,427)
test xmltree_iter_descendants_expensive     ... bench:     372,464 ns/iter (+/- 52,938)

test roxmltree_iter_descendants_inexpensive ... bench:      20,800 ns/iter (+/- 642)
test xmltree_iter_descendants_inexpensive   ... bench:     133,030 ns/iter (+/- 10,546)

test roxmltree_iter_children                ... bench:       2,168 ns/iter (+/- 12)
```

Where expensive refers to the matching done on each element. In these
benchmarks, *expensive* means searching for any node in the document which
contains a string. And *inexpensive* means searching for any element with a
particular name.

### Notes

The benchmarks were taken on a Apple M1 Pro.
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
