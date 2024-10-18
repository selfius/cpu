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

    insert_joints_into_graph(&mut graph, &nodes, &mut position_to_node)?;
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

fn insert_joints_into_graph<'a>(
    graph: &mut Graph,
    nodes: &'a [Node],
    position_to_node: &mut HashMap<&'a Position, GraphNodeRef>,
) -> Result<(), ParseError> {
    let mut wire_joints: HashMap<&Position, u32> = HashMap::default();

    let wire_ends: Vec<&Position> = nodes
        .iter()
        .flat_map(|node| -> Vec<&Position> {
            if let Node::Wire { start, end } = node {
                return vec![&start, &end];
            }
            vec![]
        })
        .collect();

    for wire_end in &wire_ends {
        wire_joints.insert(
            wire_end,
            wire_joints
                .get(wire_end)
                .map(|count| count + 1_u32)
                .unwrap_or(1),
        );
    }

    for wire_end in wire_ends {
        let n_wires_intersect = wire_joints.get(wire_end).unwrap();
        if *n_wires_intersect == 1 && position_to_node.get(wire_end).is_none() {
            return Err(ParseError::LooseWire {
                position: wire_end.clone(),
            });
        } else if *n_wires_intersect > 1 && !position_to_node.contains_key(wire_end) {
            position_to_node.insert(wire_end, graph.add_node(NodeKind::Joint));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::parse;
    use crate::types::*;

    #[test]
    fn complete_graph() {
        let test_circuit = "
                 ┏━━━┓                     
              ─┬─┨not┠─────┐               
               │ ┗━━━┛     │               
               │   ┏━━━┓   │               
              ─┼─┬─┨not┠───┼─┐             
               │ │ ┗━━━┛   │ │             
               │ │   ┏━━━┓ │ │             
              ─┼─┼─┬─┨not┠─┼─┼─┐           
               │ │ │ ┗━━━┛ │ │ │ ┏━━━━━┓   
               │ │ │       ├─┼─┼─┨ and ┃   
               │ │ │       │ ├─┼─┨ rem ┠─  
               │ │ │       │ │ └─┨     ┃   
               │ │ │       │ │   ┗━━━━━┛   
               │ │ │       │ │   ┏━━━━━┓   
               │ │ │       └─┼───┨ and ┃   
               │ │ │         └───┨     ┠─  
               │ │ ├─────────────┨     ┃   
               │ │ │             ┗━━━━━┛   
               │ │ │             ┏━━━━━┓   
               └─┼─┼─────────────┨ and ┃   
                 └─┼─────────────┨ last┠─  
                   └─────────────┨     ┃   
                                 ┗━━━━━┛   
    ";

        let graph = parse(test_circuit).unwrap();

        assert_eq!(
            format!("{graph:?}"),
            "\
            0 component_input(0) -> [ ]\n\
            1 component_output(0) -> [ ]\n\
            2 component_input(0) -> [ ]\n\
            3 component_output(0) -> [ ]\n\
            4 component_input(0) -> [ ]\n\
            5 component_output(0) -> [ ]\n\
            6 component_input(0) -> [ ]\n\
            7 component_input(1) -> [ ]\n\
            8 component_input(2) -> [ ]\n\
            9 component_output(0) -> [ ]\n\
            10 component_input(0) -> [ ]\n\
            11 component_input(1) -> [ ]\n\
            12 component_input(2) -> [ ]\n\
            13 component_output(0) -> [ ]\n\
            14 component_input(0) -> [ ]\n\
            15 component_input(1) -> [ ]\n\
            16 component_input(2) -> [ ]\n\
            17 component_output(0) -> [ ]\n\
            18 input(0) -> [ ]\n\
            19 input(1) -> [ ]\n\
            20 input(2) -> [ ]\n\
            21 output(0) -> [ ]\n\
            22 output(1) -> [ ]\n\
            23 output(2) -> [ ]\n\
            24 joint -> [ ]\n\
            25 joint -> [ ]\n\
            26 joint -> [ ]\n\
            27 joint -> [ ]\n\
            28 joint -> [ ]\n\
            29 joint -> [ ]\n\
            "
        );
    }

    #[test]
    fn detects_loose_wiring() {
        let test_circuit = "
              ───┬──┐   
                 │  └──┐
               ──┼─────┼──
                 │     └────
    ";
        let error = parse(test_circuit).unwrap_err();
        assert_eq!(
            error,
            ParseError::LooseWire {
                position: Position {
                    line: 4,
                    column: 17
                },
            },
        );
    }
}
