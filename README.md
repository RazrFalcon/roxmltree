# roxmltree
[![Build Status](https://travis-ci.org/RazrFalcon/roxmltree.svg?branch=master)](https://travis-ci.org/RazrFalcon/roxmltree)
[![Crates.io](https://img.shields.io/crates/v/roxmltree.svg)](https://crates.io/crates/roxmltree)
[![Documentation](https://docs.rs/roxmltree/badge.svg)](https://docs.rs/roxmltree)

Represent an [XML 1.0](https://www.w3.org/TR/xml/) document as a read-only tree.

## Why read-only?

Because in some cases all you need is to retrieve some data from the XML document.
And for such cases, we can make a lot of optimizations.

As for *roxmltree*, it's fast not only because it's read-only, but also because
it uses [xmlparser], which is times faster then [xml-rs].
See [Performance](#performance) section for details.

## Parsing behavior

Sadly, XML can be parsed in many different ways. The *roxmltree* is trying to mimic the
Python's [lxml](https://lxml.de/) behavior.

Unlike the *lxml*, *roxmltree* do support comments outside the root element.

Fo more details see [docs/parsing.md](https://github.com/RazrFalcon/roxmltree/blob/master/docs/parsing.md).

## Alternatives

\* Rust besed for now

| Feature/Crate                   | roxmltree        | [xmltree]        | [elementtree]    | [sxd-document]   | [treexml]        |
| ------------------------------- | :--------------: | :--------------: | :--------------: | :--------------: | :--------------: |
| Element namespace resolving     | ✔                | ✔                | ✔               | ~<sup>1</sup>     |                  |
| Attribute namespace resolving   | ✔                |                  |                  | ✔                |                  |
| [Entity references]             | ✔<sup>2</sup>    | ⚠                | ⚠                | ⚠             | ⚠                |
| [Character references]          | ✔                | ✔                | ✔                | ✔                | ✔                |
| [Attribute-Value normalization] | ✔                |                  |                  |                  |                  |
| Comments                        | ✔                |                  |                  | ✔                |                  |
| Processing instructions         | ✔                | ⚠                |                  | ✔               |                  |
| UTF-8 BOM                       | ✔                | ⚠               | ⚠               | ⚠               | ⚠                |
| Non UTF-8 input                 |                  |                  |                  |                  |                  |
| Complete DTD support            |                  |                  |                  |                  |                  |
| Position preserving<sup>3</sup> | ✔                |                 |                 |                 |                  |
| `xml:space`                     |                  |                  |                  |                  |                  |
| Tree modifications              |                  | ✔                | ✔                | ✔                | ✔                |
| Writing                         |                  | ✔                | ✔                | ✔                | ✔                |
| No **unsafe**                   | ✔                | ✔                | ~<sup>4</sup>    |                  | ✔                |
| Size overhead<sup>5</sup>       | **~60KiB**       | ~80KiB           | ~96KiB           | ~130KiB          | ~110KiB          |
| Dependencies                    | **1**            | 2                | 18               | 2                | 14               |
| Tested version                  | 0.1.0            | 0.8.0            | 0.5.0            | 0.2.6            | 0.7.0            |
| License                         | MIT / Apache-2.0 | MIT              | BSD-3-Clause     | MIT              | MIT              |

Legend:

- ✔ - supported
- ⚠ - parsing error
- ~ - partial
- *nothing* - not supported

Notes:

1. No default namespace propagation.
2. Nested/indirect entity references are not supported yet.
3. *roxmltree* keeps all nodes and attributes position in the original document,
   so you can easily retrieve it if you need it.
   See [examples/print_pos.rs](examples/print_pos.rs) for details.
4. In the `string_cache` crate.
5. Binary size overhead according to the [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat).

[Entity references]: https://www.w3.org/TR/REC-xml/#dt-entref
[Character references]: https://www.w3.org/TR/REC-xml/#NT-CharRef
[Attribute-Value Normalization]: https://www.w3.org/TR/REC-xml/#AVNormalize

[xmltree]: https://crates.io/crates/xmltree
[elementtree]: https://crates.io/crates/elementtree
[treexml]: https://crates.io/crates/treexml
[sxd-document]: https://crates.io/crates/sxd-document

## Performance

```text
test large_roxmltree     ... bench:   8,807,741 ns/iter (+/- 70,532)
test large_sdx_document  ... bench:   9,777,811 ns/iter (+/- 242,912)
test large_xmltree       ... bench:  31,041,407 ns/iter (+/- 27,171)
test large_treexml       ... bench:  32,048,129 ns/iter (+/- 29,860)
test large_elementtree   ... bench:  32,073,296 ns/iter (+/- 68,433)

test medium_roxmltree    ... bench:   1,735,369 ns/iter (+/- 3,218)
test medium_sdx_document ... bench:   3,569,814 ns/iter (+/- 10,518)
test medium_treexml      ... bench:  11,163,737 ns/iter (+/- 26,084)
test medium_xmltree      ... bench:  11,267,754 ns/iter (+/- 70,971)
test medium_elementtree  ... bench:  11,629,513 ns/iter (+/- 27,055)
```

*roxmltree* uses [xmlparser] internally,
while *sdx-document* uses it's own one and *xmltree*, *elementtree* and *treexml* are using the
[xml-rs] crate.
Here is a comparison between *xmlparser* and *xml-rs*:

```text
test large_xmlparser     ... bench:   2,019,245 ns/iter (+/- 693)
test large_xmlrs         ... bench:  29,086,480 ns/iter (+/- 22,741)

test medium_xmlparser    ... bench:     434,140 ns/iter (+/- 231)
test medium_xmlrs        ... bench:  10,391,411 ns/iter (+/- 24,738)
```

*Note:* tree crates may use different *xml-rs* crate versions.

You can try it yourself using `cargo bench --features benchmark`

[xml-rs]: https://crates.io/crates/xml-rs
[xmlparser]: https://crates.io/crates/xmlparser

## Safety

- The library must not panic. Any panic considered as a critical bug
  and should be reported.
- The library forbids the unsafe code.

## Non-goals

- Complete XML support.
- Tree modification and writing.
- XPath/XQuery.

## API

The library uses Rust's idiomatic API based on iterators.
In case you are more familiar with the browsers/JS DOM API - you can check out
the [tests/dom-api.rs](tests/dom-api.rs) to see how it can be converted into a Rust one.

## Dependency

[Rust](https://www.rust-lang.org/) >= 1.18

## License

Licensed under either of

- [Apache License v2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
