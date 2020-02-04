use petgraph::graph::Graph;
use petgraph_graphml::GraphMl;

#[test]
fn single_node() {
    let mut deps = Graph::<&str, &str>::new();
    deps.add_node("petgraph");

    let graphml = GraphMl::new(&deps).pretty_print(true);
    let xml = graphml.to_string();
    let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0" />
  </graph>
</graphml>"#;

    assert_eq!(expected, xml);
}

#[test]
fn single_node_disable_pretty() {
    let mut deps = Graph::<&str, &str>::new();
    deps.add_node("petgraph");

    let graphml = GraphMl::new(&deps).pretty_print(false);
    let xml = graphml.to_string();
    let expected = r#"<?xml version="1.0" encoding="UTF-8"?><graphml xmlns="http://graphml.graphdrawing.org/xmlns"><graph edgedefault="directed"><node id="n0" /></graph></graphml>"#;

    assert_eq!(expected, xml);
}

#[test]
fn single_node_with_display_weight() {
    let mut deps = Graph::<&str, &str>::new();
    deps.add_node("petgraph");

    let graphml = GraphMl::new(&deps)
        .pretty_print(true)
        .export_node_weights_display();
    let xml = graphml.to_string();
    let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0">
      <data key="weight">petgraph</data>
    </node>
  </graph>
  <key id="weight" for="node" attr.name="weight" attr.type="string" />
</graphml>"#;

    assert_eq!(expected, xml);
}

#[test]
fn single_edge() {
    let mut deps = Graph::<&str, &str>::new();
    let pg = deps.add_node("petgraph");
    let fb = deps.add_node("fixedbitset");
    deps.extend_with_edges(&[(pg, fb)]);

    let graphml = GraphMl::new(&deps).pretty_print(true);
    let xml = graphml.to_string();
    let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0" />
    <node id="n1" />
    <edge id="e0" source="n0" target="n1" />
  </graph>
</graphml>"#;
    assert_eq!(expected, xml);
}

#[test]
fn single_edge_with_display_weight() {
    let mut deps = Graph::<&str, &str>::new();
    let pg = deps.add_node("petgraph");
    let fb = deps.add_node("fixedbitset");
    deps.update_edge(pg, fb, "depends on");

    let graphml = GraphMl::new(&deps)
        .pretty_print(true)
        .export_edge_weights_display();
    let xml = graphml.to_string();
    let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0" />
    <node id="n1" />
    <edge id="e0" source="n0" target="n1">
      <data key="weight">depends on</data>
    </edge>
  </graph>
  <key id="weight" for="edge" attr.name="weight" attr.type="string" />
</graphml>"#;
    assert_eq!(expected, xml);
}

#[test]
fn node_and_edge_display_weight() {
    let mut deps = Graph::<&str, &str>::new();
    let pg = deps.add_node("petgraph");
    let fb = deps.add_node("fixedbitset");
    deps.update_edge(pg, fb, "depends on");

    let graphml = GraphMl::new(&deps)
        .pretty_print(true)
        .export_edge_weights_display()
        .export_node_weights_display();
    let xml = graphml.to_string();
    let expected1 = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph edgedefault="directed">
    <node id="n0">
      <data key="weight">petgraph</data>
    </node>
    <node id="n1">
      <data key="weight">fixedbitset</data>
    </node>
    <edge id="e0" source="n0" target="n1">
      <data key="weight">depends on</data>
    </edge>
  </graph>"#;
    let expected2 = r#"<key id="weight" for="edge" attr.name="weight" attr.type="string" />"#;
    let expected3 = r#"<key id="weight" for="node" attr.name="weight" attr.type="string" />"#;
    let expected4 = r#"</graphml>"#;

    // HashSet output is unordered, therefore we do not know the order of the keys
    assert!(xml.starts_with(expected1));
    assert!(xml.contains(expected2));
    assert!(xml.contains(expected3));
    assert!(xml.ends_with(expected4));
}
