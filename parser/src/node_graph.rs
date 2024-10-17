use crate::types::{Node, ParseError, Position};
use core::ops::Range;
use digital_component::{DigitalComponent, Graph, GraphNodeRef, NodeKind};
use std::collections::HashMap;
use std::rc::Rc;

fn create_component_from_text_nodes(_: Vec<&Node>) -> DigitalComponent {
    DigitalComponent::new(1, 1, Box::new(|_, _| false))
}

pub fn build_node_graph(mut nodes: Vec<Node>) -> Result<Graph, ParseError> {
    let mut graph = Graph::default();
    let mut position_to_node: HashMap<&Position, GraphNodeRef> = HashMap::default();
    nodes.sort_by_key(|node| node.sort_key());

    for (box_node, text_nodes) in correlate_boxes_and_text(&nodes) {
        //TODO based on the node we and token we can create a digital component
        if let Node::Box {
            inputs, outputs, ..
        } = box_node
        {
            let component = Rc::new(create_component_from_text_nodes(text_nodes));

            for (idx, input_position) in inputs.iter().enumerate() {
                let node_ref = graph.add_node(NodeKind::ComponentInput {
                    component: Rc::clone(&component),
                    input: idx,
                });
                position_to_node.insert(input_position, node_ref);
            }

            for (idx, output_position) in outputs.iter().enumerate() {
                let node_ref = graph.add_node(NodeKind::ComponentOutput {
                    component: Rc::clone(&component),
                    output: idx,
                });
                position_to_node.insert(output_position, node_ref);
            }
        } else {
            panic!("node {:?} is always expected to be a box node", box_node);
        }
    }

    insert_inputs_outputs_into_graph(&mut graph, &nodes, &mut position_to_node);

    // TODO find joints and put them into the graph and the map as well
    // TODO go through wires and treat them as connections

    println!("{:?}", graph);
    Ok(graph)
}

fn correlate_boxes_and_text(nodes: &Vec<Node>) -> Vec<(&Node, Vec<&Node>)> {
    //TODO this can be turned into n log n with a kd-tree
    let mut result = vec![];
    for node in nodes {
        if let Node::Box {
            top_left: box_top_left,
            bottom_right: box_bottom_right,
            ..
        } = node
        {
            let mut text_tokens = vec![];
            for text_node in nodes {
                if let Node::Text {
                    line,
                    position: Range { start: column, .. },
                    ..
                } = text_node
                {
                    if *line > box_top_left.line
                        && *line < box_bottom_right.line
                        && *column > box_top_left.column
                        && *column < box_bottom_right.column
                    {
                        text_tokens.push(text_node);
                    }
                }
            }
            result.push((node, text_tokens));
        }
    }
    result
}

fn insert_inputs_outputs_into_graph<'a>(
    graph: &mut Graph,
    nodes: &'a [Node],
    position_to_node: &mut HashMap<&'a Position, GraphNodeRef>,
) {
    let mut input_idx = 0_usize;
    let mut output_idx = 0_usize;
    for node in nodes {
        match node {
            Node::Input { position } => {
                position_to_node.insert(position, graph.add_node(NodeKind::Input(input_idx)));
                input_idx += 1;
            }

            Node::Output { position } => {
                position_to_node.insert(position, graph.add_node(NodeKind::Output(output_idx)));
                output_idx += 1;
            }
            _ => {}
        };
    }
}
