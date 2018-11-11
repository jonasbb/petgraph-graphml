//! [![docs.rs badge](https://docs.rs/petgraph-graphml/badge.svg)](https://docs.rs/petgraph-graphml/)
//! [![crates.io badge](https://img.shields.io/crates/v/petgraph-graphml.svg)](https://crates.io/crates/petgraph-graphml/)
//! [![Build Status](https://travis-ci.com/jonasbb/petgraph-graphml.svg?branch=master)](https://travis-ci.com/jonasbb/petgraph-graphml)
//! [![codecov](https://codecov.io/gh/jonasbb/petgraph-graphml/branch/master/graph/badge.svg)](https://codecov.io/gh/jonasbb/petgraph-graphml)
//!
//! ---
//!
//! This crate extends [petgraph][] with [GraphML][graphmlwebsite] output support.
//!
//! This crate exports a single type [`GraphMl`] which combines a build-pattern for configuration and provides creating strings ([`GraphMl::to_string`]) and writing to writers ([`GraphMl::to_writer`]).
//!
//! # Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! petgraph-graphml = "1.0.0"
//! ```
//!
//! # Example
//!
//! For a simple graph like ![Graph with three nodes and two edges](https://github.com/jonasbb/petgraph-graphml/tree/master/doc/graph.png) this is the generated GraphML output.
//!
//! ```
//! # extern crate petgraph;
//! # extern crate petgraph_graphml;
//! # use petgraph::Graph;
//! # use petgraph_graphml::GraphMl;
//! # fn make_graph() -> Graph<u32, ()> {
//! #     let mut graph = Graph::new();
//! #     let n0 = graph.add_node(0);
//! #     let n1 = graph.add_node(1);
//! #     let n2 = graph.add_node(2);
//! #     graph.update_edge(n0, n1, ());
//! #     graph.update_edge(n1, n2, ());
//! #     graph
//! # }
//! # fn main() {
//! let graph = make_graph();
//! // Configure output settings
//! // Enable pretty printing and exporting of node weights.
//! // Use the Display implementation of NodeWeights for exporting them.
//! let graphml = GraphMl::new(&graph)
//!     .pretty_print(true)
//!     .export_node_weights_display();
//!
//! assert_eq!(
//!     graphml.to_string(),
//!     r#"<?xml version="1.0" encoding="UTF-8"?>
//! <graphml xmlns="http://graphml.graphdrawing.org/xmlns">
//!   <graph edgedefault="directed">
//!     <node id="n0">
//!       <data key="weight">0</data>
//!     </node>
//!     <node id="n1">
//!       <data key="weight">1</data>
//!     </node>
//!     <node id="n2">
//!       <data key="weight">2</data>
//!     </node>
//!     <edge id="e0" source="n0" target="n1" />
//!     <edge id="e1" source="n1" target="n2" />
//!   </graph>
//!   <key id="weight" for="node" attr.name="weight" attr.type="string" />
//! </graphml>"#
//! );
//! # }
//! ```
//!
//! [`GraphMl`]: https://docs.rs/petgraph-graphml/*/petgraph_graphml/struct.GraphMl.html
//! [`GraphMl::to_string`]: https://docs.rs/petgraph-graphml/*/petgraph_graphml/struct.GraphMl.html#method.to_string
//! [`GraphMl::to_writer`]: https://docs.rs/petgraph-graphml/*/petgraph_graphml/struct.GraphMl.html#method.to_writer
//! [graphmlwebsite]: http://graphml.graphdrawing.org/
//! [petgraph]: https://docs.rs/petgraph/
//!

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]

extern crate petgraph;
extern crate xml;

use petgraph::visit::{
    EdgeRef, GraphProp, IntoEdgeReferences, IntoNodeReferences, NodeIndexable, NodeRef,
};
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Debug;
use std::io::{self, Cursor, Write};
use std::string::ToString;
use xml::common::XmlVersion;
use xml::writer::events::XmlEvent;
use xml::writer::{Error as XmlError, EventWriter, Result as WriterResult};
use xml::EmitterConfig;

static NAMESPACE_URL: &str = "http://graphml.graphdrawing.org/xmlns";

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct Attribute {
    name: Cow<'static, str>,
    for_: For,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum For {
    Node,
    Edge,
}

impl For {
    fn to_str(self) -> &'static str {
        match self {
            For::Node => "node",
            For::Edge => "edge",
        }
    }
}

type PrintWeights<W> = for<'a> Fn(&'a W) -> Vec<(Cow<'static, str>, Cow<'a, str>)>;

/// GraphML output printer
///
/// See the [main crate documentation](index.html) for usage instructions and examples.
pub struct GraphMl<G>
where
    G: IntoEdgeReferences,
    G: IntoNodeReferences,
{
    graph: G,
    pretty_print: bool,
    export_edges: Option<Box<PrintWeights<G::EdgeWeight>>>,
    export_nodes: Option<Box<PrintWeights<G::NodeWeight>>>,
}

impl<G> GraphMl<G>
where
    G: GraphProp,
    G: IntoNodeReferences,
    G: IntoEdgeReferences,
    G: NodeIndexable,
{
    /// Create a new GraphML printer for the graph.
    pub fn new(graph: G) -> Self {
        Self {
            graph,
            pretty_print: true,
            export_edges: None,
            export_nodes: None,
        }
    }

    /// Enable or disble pretty printing of the XML.
    ///
    /// Pretty printing enables linebreaks and indentation.
    pub fn pretty_print(mut self, state: bool) -> Self {
        self.pretty_print = state;
        self
    }

    /// Export the edge weights to GraphML.
    ///
    /// This uses the [`Display`] implementation of the edge weight type.
    /// The attribute name defaults to "weight".
    ///
    /// Once set this option cannot be disabled anymore.
    ///
    /// [`Display`]: ::std::fmt::Display
    pub fn export_edge_weights_display(self) -> Self
    where
        G::EdgeWeight: ToString,
    {
        self.export_edge_weights(Box::new(|edge| {
            vec![("weight".into(), edge.to_string().into())]
        }))
    }

    /// Export the edge weights to GraphML.
    ///
    /// This uses a custom conversion function.
    /// Each edge can be converted into an arbitray number of attributes.
    /// Each attribute is a key-value pair, represented as tuple.
    ///
    /// Once set this option cannot be disabled anymore.
    ///
    /// # Example
    ///
    /// A custom print function for the type `(String, u32)`.
    /// It will create two attributes "str attr" and "int attr" containing the string and integer part.
    ///
    /// ```
    /// # extern crate petgraph;
    /// # extern crate petgraph_graphml;
    /// # use petgraph::Graph;
    /// # use petgraph_graphml::GraphMl;
    /// # fn make_graph() -> Graph<(), (String, u32)> {
    /// #     Graph::new()
    /// # }
    /// # fn main() {
    /// let graph = make_graph();
    /// let graphml = GraphMl::new(&graph)
    ///     .export_edge_weights(Box::new(|edge| {
    ///         let &(ref s, i) = edge;
    ///         vec![
    ///             ("str attr".into(), s[..].into()),
    ///             ("int attr".into(), i.to_string().into()),
    ///         ]
    ///     }));
    /// # }
    /// ```
    ///
    /// Currently only string attribute types are supported.
    pub fn export_edge_weights(mut self, edge_weight: Box<PrintWeights<G::EdgeWeight>>) -> Self {
        self.export_edges = Some(edge_weight);
        self
    }

    /// Export the node weights to GraphML.
    ///
    /// This uses the [`Display`] implementation of the node weight type.
    /// The attribute name defaults to "weight".
    ///
    /// Once set this option cannot be disabled anymore.
    ///
    /// [`Display`]: ::std::fmt::Display
    pub fn export_node_weights_display(self) -> Self
    where
        G::NodeWeight: ToString,
    {
        self.export_node_weights(Box::new(|node| {
            vec![("weight".into(), node.to_string().into())]
        }))
    }

    /// Export the node weights to GraphML.
    ///
    /// This uses a custom conversion function.
    /// Each node can be converted into an arbitray number of attributes.
    /// Each attribute is a key-value pair, represented as tuple.
    ///
    /// Once set this option cannot be disabled anymore.
    ///
    /// # Example
    ///
    /// A custom print function for the type `(String, u32)`.
    /// It will create two attributes "str attr" and "int attr" containing the string and integer part.
    ///
    /// ```
    /// # extern crate petgraph;
    /// # extern crate petgraph_graphml;
    /// # use petgraph::Graph;
    /// # use petgraph_graphml::GraphMl;
    /// # fn make_graph() -> Graph<(String, u32), ()> {
    /// #     Graph::new()
    /// # }
    /// # fn main() {
    /// let graph = make_graph();
    /// let graphml = GraphMl::new(&graph)
    ///     .export_node_weights(Box::new(|node| {
    ///         let &(ref s, i) = node;
    ///         vec![
    ///             ("str attr".into(), s[..].into()),
    ///             ("int attr".into(), i.to_string().into()),
    ///         ]
    ///     }));
    /// # }
    /// ```
    ///
    /// Currently only string attribute types are supported.
    pub fn export_node_weights(mut self, node_weight: Box<PrintWeights<G::NodeWeight>>) -> Self {
        self.export_nodes = Some(node_weight);
        self
    }

    /// Create a string with the GraphML content.
    pub fn to_string(&self) -> String {
        let mut buff = Cursor::new(Vec::new());
        self.to_writer(&mut buff)
            .expect("Writing to a Cursor should never create IO errors.");
        String::from_utf8(buff.into_inner()).unwrap()
    }

    /// Write the GraphML file to the given writer.
    pub fn to_writer<W>(&self, writer: W) -> io::Result<()>
    where
        W: Write,
    {
        let mut writer = EventWriter::new_with_config(
            writer,
            EmitterConfig::new().perform_indent(self.pretty_print),
        );
        match self.emit_graphml(&mut writer) {
            Ok(()) => Ok(()),
            Err(XmlError::Io(ioerror)) => Err(ioerror),
            _ => panic!(""),
        }
    }

    fn emit_graphml<W>(&self, writer: &mut EventWriter<W>) -> WriterResult<()>
    where
        W: Write,
    {
        // Store information about the attributes for nodes and edges.
        // We cannot know in advance what the attribute names will be, so we just keep track of what gets emitted.
        let mut attributes: HashSet<Attribute> = HashSet::new();

        // XML/GraphML boilerplate
        writer.write(XmlEvent::StartDocument {
            version: XmlVersion::Version10,
            encoding: Some("UTF-8"),
            standalone: None,
        })?;
        writer.write(XmlEvent::start_element("graphml").attr("xmlns", NAMESPACE_URL))?;

        // emit graph with nodes/edges and possibly weights
        self.emit_graph(writer, &mut attributes)?;
        // Emit <key> tags for all the attributes
        self.emit_keys(writer, &attributes)?;

        writer.write(XmlEvent::end_element())?; // end graphml
        Ok(())
    }

    fn emit_graph<W>(
        &self,
        writer: &mut EventWriter<W>,
        attributes: &mut HashSet<Attribute>,
    ) -> WriterResult<()>
    where
        W: Write,
    {
        // convenience function to turn a NodeId into a String
        let node2str_id = |node: G::NodeId| -> String { format!("n{}", self.graph.to_index(node)) };
        // Emit an attribute for either node or edge
        // This will also keep track of updating the global attributes list
        let mut emit_attribute = |writer: &mut EventWriter<_>,
                                  name: Cow<'static, str>,
                                  data: &str,
                                  for_: For|
         -> WriterResult<()> {
            writer.write(XmlEvent::start_element("data").attr("key", &*name))?;
            attributes.insert(Attribute { name, for_ });
            writer.write(XmlEvent::characters(data))?;
            writer.write(XmlEvent::end_element()) // end data
        };

        // Each graph needs a default edge type
        writer.write(XmlEvent::start_element("graph").attr(
            "edgedefault",
            if self.graph.is_directed() {
                "directed"
            } else {
                "undirected"
            },
        ))?;

        // Emit nodes
        for node in self.graph.node_references() {
            writer.write(XmlEvent::start_element("node").attr("id", &*node2str_id(node.id())))?;
            // Print weights
            if let Some(ref node_labels) = self.export_nodes {
                let datas = node_labels(&node.weight());
                for (name, data) in datas {
                    emit_attribute(writer, name, &*data, For::Node)?;
                }
            }
            writer.write(XmlEvent::end_element())?; // end node
        }

        // Emit edges
        for (i, edge) in self.graph.edge_references().enumerate() {
            writer.write(
                XmlEvent::start_element("edge")
                    .attr("id", &format!("e{}", i))
                    .attr("source", &*node2str_id(edge.source()))
                    .attr("target", &*node2str_id(edge.target())),
            )?;
            // Print weights
            if let Some(ref edge_labels) = self.export_edges {
                let datas = edge_labels(&edge.weight());
                for (name, data) in datas {
                    emit_attribute(writer, name, &*data, For::Edge)?;
                }
            }
            writer.write(XmlEvent::end_element())?; // end edge
        }
        writer.write(XmlEvent::end_element()) // end graph
    }

    fn emit_keys<W>(
        &self,
        writer: &mut EventWriter<W>,
        attributes: &HashSet<Attribute>,
    ) -> WriterResult<()>
    where
        W: Write,
    {
        for attr in attributes {
            writer.write(
                XmlEvent::start_element("key")
                    .attr("id", &*attr.name)
                    .attr("for", attr.for_.to_str())
                    .attr("attr.name", &*attr.name)
                    .attr("attr.type", "string"),
            )?;
            writer.write(XmlEvent::end_element())?; // end key
        }
        Ok(())
    }
}

impl<G> Debug for GraphMl<G>
where
    G: Debug,
    G: IntoEdgeReferences,
    G: IntoNodeReferences,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("GraphMl")
            .field("graph", &self.graph)
            .field("pretty_print", &self.pretty_print)
            .field("export_edges", &self.export_edges.is_some())
            .field("export_nodes", &self.export_nodes.is_some())
            .finish()
    }
}
