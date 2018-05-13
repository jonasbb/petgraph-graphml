#![deny(
    missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unused_extern_crates, unused_import_braces, unused_qualifications,
    variant_size_differences
)]
#![warn(missing_docs)]

//! Simple graphml file format output.

extern crate petgraph;
extern crate xml;

use petgraph::visit::{
    EdgeRef, GraphProp, IntoEdgeReferences, IntoNodeReferences, NodeIndexable, NodeRef,
};
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Debug;
use std::io::{Cursor, Result as IoResult, Write};
use std::string::ToString;
use xml::common::XmlVersion;
use xml::writer::events::XmlEvent;
use xml::writer::Error as XmlError;
use xml::writer::{EventWriter, Result as WriterResult};
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
    fn to_str(&self) -> &'static str {
        match *self {
            For::Node => "node",
            For::Edge => "edge",
        }
    }
}

type PrintWeights<W> = for<'a> Fn(&'a W) -> Vec<(Cow<'static, str>, Cow<'a, str>)>;

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
    pub fn new(graph: G) -> Self {
        Self {
            graph,
            pretty_print: true,
            export_edges: None,
            export_nodes: None,
        }
    }

    pub fn pretty_print(mut self, state: bool) -> Self {
        self.pretty_print = state;
        self
    }

    pub fn export_edge_weights_display(self) -> Self
    where
        G::EdgeWeight: ToString,
    {
        self.export_edge_weights(Box::new(|edge| {
            vec![("weight".into(), edge.to_string().into())]
        }))
    }

    pub fn export_edge_weights(mut self, edge_weight: Box<PrintWeights<G::EdgeWeight>>) -> Self {
        self.export_edges = Some(edge_weight);
        self
    }

    pub fn export_node_weights_display(self) -> Self
    where
        G::NodeWeight: ToString,
    {
        self.export_node_weights(Box::new(|node| {
            vec![("weight".into(), node.to_string().into())]
        }))
    }

    pub fn export_node_weights(mut self, node_weight: Box<PrintWeights<G::NodeWeight>>) -> Self {
        self.export_nodes = Some(node_weight);
        self
    }

    pub fn to_string(&self) -> String {
        let mut buff = Cursor::new(Vec::new());
        self.to_writer(&mut buff)
            .expect("Writing to a Cursor should never create IO errors.");
        String::from_utf8(buff.into_inner()).unwrap()
    }

    pub fn to_writer<W>(&self, writer: W) -> IoResult<()>
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
