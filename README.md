# [GraphML][graphmlwebsite] output support for [petgraph]

[![docs.rs badge](https://docs.rs/petgraph-graphml/badge.svg)](https://docs.rs/petgraph-graphml/)
[![crates.io badge](https://img.shields.io/crates/v/petgraph-graphml.svg)](https://crates.io/crates/petgraph-graphml/)
[![Build Status](https://travis-ci.com/jonasbb/petgraph-graphml.svg?branch=master)](https://travis-ci.com/jonasbb/petgraph-graphml)
[![codecov](https://codecov.io/gh/jonasbb/petgraph-graphml/branch/master/graph/badge.svg)](https://codecov.io/gh/jonasbb/petgraph-graphml)

---

This crate extends [petgraph][] with [GraphML][graphmlwebsite] output support.

This crate exports a single type [`GraphMl`] which combines a build-pattern for configuration and provides creating strings ([`GraphMl::to_string`]) and writing to writers ([`GraphMl::to_writer`]).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
petgraph-graphml = "1.0.0"
```

## Example

For a simple graph like ![Graph with three nodes and two edges](https://github.com/jonasbb/petgraph-graphml/tree/master/doc/graph.png) this is the generated GraphML output.

```rust
let graph = make_graph();
// Configure output settings
// Enable pretty printing and exporting of node weights.
// Use the Display implementation of NodeWeights for exporting them.
let graphml = GraphMl::new(&graph)
    .pretty_print(true)
    .export_node_weights_display();

assert_eq!(
    graphml.to_string(),
    r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0">
      <data key="weight">0</data>
    </node>
    <node id="n1">
      <data key="weight">1</data>
    </node>
    <node id="n2">
      <data key="weight">2</data>
    </node>
    <edge id="e0" source="n0" target="n1" />
    <edge id="e1" source="n1" target="n2" />
  </graph>
  <key id="weight" for="node" attr.name="weight" attr.type="string" />
</graphml>"#
);
```

[`GraphMl`]: https://docs.rs/petgraph-graphml/*/petgraph_graphml/struct.GraphMl.html
[`GraphMl::to_string`]: https://docs.rs/petgraph-graphml/*/petgraph_graphml/struct.GraphMl.html#method.to_string
[`GraphMl::to_writer`]: https://docs.rs/petgraph-graphml/*/petgraph_graphml/struct.GraphMl.html#method.to_writer
[graphmlwebsite]: http://graphml.graphdrawing.org/
[petgraph]: https://docs.rs/petgraph/

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
