# roxmltree
[![Build Status](https://travis-ci.org/RazrFalcon/roxmltree.svg?branch=master)](https://travis-ci.org/RazrFalcon/roxmltree)
[![Crates.io](https://img.shields.io/crates/v/roxmltree.svg)](https://crates.io/crates/roxmltree)
[![Documentation](https://docs.rs/roxmltree/badge.svg)](https://docs.rs/roxmltree)
[![Rust 1.18+](https://img.shields.io/badge/rust-1.18+-orange.svg)](https://www.rust-lang.org)

Represents an [XML 1.0](https://www.w3.org/TR/xml/) document as a read-only tree.

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
| Tested version                  | 0.7.0            | 2.9.8               | 0.8.0            | 0.5.0            | 0.3.0            | 0.7.0            |
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
test large_roxmltree     ... bench:   5,575,373 ns/iter (+/- 54,817)
test large_sdx_document  ... bench:   9,452,579 ns/iter (+/- 37,298)
test large_xmltree       ... bench:  28,383,408 ns/iter (+/- 46,793)
test large_treexml       ... bench:  28,992,626 ns/iter (+/- 122,244)
test large_elementtree   ... bench:  29,991,730 ns/iter (+/- 58,134)

test medium_roxmltree    ... bench:   1,085,323 ns/iter (+/- 32,921)
test medium_sdx_document ... bench:   3,619,042 ns/iter (+/- 8,863)
test medium_xmltree      ... bench:  10,181,629 ns/iter (+/- 13,994)
test medium_treexml      ... bench:  10,338,760 ns/iter (+/- 11,040)
test medium_elementtree  ... bench:  10,840,762 ns/iter (+/- 16,162)

test tiny_roxmltree      ... bench:       7,381 ns/iter (+/- 34)
test tiny_sdx_document   ... bench:      27,464 ns/iter (+/- 91)
test tiny_xmltree        ... bench:      43,838 ns/iter (+/- 107)
test tiny_treexml        ... bench:      44,794 ns/iter (+/- 263)
test tiny_elementtree    ... bench:      45,431 ns/iter (+/- 175)
```

*roxmltree* uses [xmlparser] internally,
while *sdx-document* uses its own implementation and *xmltree*, *elementtree*
and *treexml* use the [xml-rs] crate.
Here is a comparison between *xmlparser* and *xml-rs*:

```text
test large_xmlparser     ... bench:   2,349,936 ns/iter (+/- 17,752)
test large_xmlrs         ... bench:  25,582,284 ns/iter (+/- 76,500)

test medium_xmlparser    ... bench:     558,500 ns/iter (+/- 315)
test medium_xmlrs        ... bench:   9,368,598 ns/iter (+/- 10,995)

test tiny_xmlparser      ... bench:       4,712 ns/iter (+/- 14)
test tiny_xmlrs          ... bench:      39,293 ns/iter (+/- 63)
```

You can try it yourself by running `cargo bench` in the `benches` dir.

Notes:

- Since all libraries have a different XML support, benchmarking is a bit pointless.
- Tree crates may use different *xml-rs* crate versions.
- We do not bench the libxml2, because `xmlReadFile()` will parse only an XML structure,
  without attributes normalization and stuff. So it's hard to compare.
  And we have to use a separate benchmark utility.

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
