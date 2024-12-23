use crate::types::{Node, ParseError, Position};
use core::ops::Range;
use digital_component::{
    ComponentInput, ComponentLogicFactory, ComponentOutput, DigitalComponent, Graph, GraphNodeRef,
    NodeKind,
};
use std::collections::HashMap;

fn create_component_from_text_nodes(
    text_nodes: Vec<&Node>,
    comp_funcs: &HashMap<&str, Box<ComponentLogicFactory>>,
) -> DigitalComponent {
    if let Node::Text { value, .. } = text_nodes[0] {
        let comp_logic = (*comp_funcs.get(&value[..]).unwrap())();
        DigitalComponent::new(2, 1, Box::new(comp_logic))
    } else {
        panic!("Expected function name got {:?}", text_nodes[0]);
    }
}

pub fn build_node_graph(
    mut nodes: Vec<Node>,
    comp_funcs: &HashMap<&str, Box<ComponentLogicFactory>>,
) -> Result<Graph, ParseError> {
    let mut graph = Graph::default();
    let mut position_to_node: HashMap<&Position, GraphNodeRef> = HashMap::default();
    nodes.sort_by_key(|node| node.sort_key());

    for (box_node, text_nodes) in correlate_boxes_and_text(&nodes) {
        if let Node::Box {
            inputs, outputs, ..
        } = box_node
        {
            let component =
                graph.add_component(create_component_from_text_nodes(text_nodes, comp_funcs));

            for (idx, input_position) in inputs.iter().enumerate() {
                let node_ref = graph.add_node(NodeKind::ComponentInput(ComponentInput::new(
                    component, idx,
                )));
                position_to_node.insert(input_position, node_ref);
            }

            for (idx, output_position) in outputs.iter().enumerate() {
                let node_ref = graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(
                    component, idx,
                )));
                position_to_node.insert(output_position, node_ref);
            }
        } else {
            panic!("node {:?} is always expected to be a box node", box_node);
        }
    }

    insert_inputs_outputs_into_graph(&mut graph, &nodes, &mut position_to_node);

    insert_joints_into_graph(&mut graph, &nodes, &mut position_to_node)?;

    add_edges(&mut graph, &nodes, &mut position_to_node);

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
                .map(|count| count + 1)
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

fn add_edges<'a>(
    graph: &mut Graph,
    nodes: &'a [Node],
    position_to_node: &mut HashMap<&'a Position, GraphNodeRef>,
) {
    for node in nodes {
        if let Node::Wire { start, end } = node {
            let a = position_to_node.get(start).unwrap();
            let b = position_to_node.get(end).unwrap();
            graph.add_edge(a, b);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;
    use crate::types::*;
    use digital_component::*;

    use std::collections::HashMap;

    fn test() -> Box<ComponentLogic> {
        Box::new(|_: &[BitState], _: &mut [BitState]| {})
    }

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

        let mut comps: HashMap<&str, Box<ComponentLogicFactory>> = HashMap::new();
        comps.insert("and", Box::new(test));
        comps.insert("not", Box::new(test));
        let graph = parse(test_circuit, &comps).unwrap();

        //       ┏━━━┓
        //  18─24┨0 1┠─────┐
        //     │ ┗━━━┛     │
        //     │   ┏━━━┓   │
        //  19─┼─26┨2 3┠───┼─┐
        //     │ │ ┗━━━┛   │ │
        //     │ │   ┏━━━┓ │ │
        //  20─┼─┼─28┨4 5┠─┼─┼─┐
        //     │ │ │ ┗━━━┛ │ │ │ ┏━━━━━┓
        //     │ │ │       25┼─┼─┨6    ┃
        //     │ │ │       │ 27┼─┨7   9┠─21
        //     │ │ │       │ │ └─┨8    ┃
        //     │ │ │       │ │   ┗━━━━━┛
        //     │ │ │       │ │   ┏━━━━━┓
        //     │ │ │       └─┼───┨10   ┃
        //     │ │ │         └───┨11 13┠─22
        //     │ │ 29────────────┨12   ┃
        //     │ │ │             ┗━━━━━┛
        //     │ │ │             ┏━━━━━┓
        //     └─┼─┼─────────────┨14   ┃
        //       └─┼─────────────┨15 17┠─23
        //         └─────────────┨16   ┃
        //                       ┗━━━━━┛

        assert_eq!(
            format!("{graph:?}"),
            "\
            0_component_input(0 0) -> [ 24_joint]\n\
            1_component_output(0 0) -> [ 25_joint]\n\
            2_component_input(1 0) -> [ 26_joint]\n\
            3_component_output(1 0) -> [ 27_joint]\n\
            4_component_input(2 0) -> [ 28_joint]\n\
            5_component_output(2 0) -> [ 8_component_input(3 2)]\n\
            6_component_input(3 0) -> [ 25_joint]\n\
            7_component_input(3 1) -> [ 27_joint]\n\
            8_component_input(3 2) -> [ 5_component_output(2 0)]\n\
            9_component_output(3 0) -> [ 21_output(0)]\n\
            10_component_input(4 0) -> [ 25_joint]\n\
            11_component_input(4 1) -> [ 27_joint]\n\
            12_component_input(4 2) -> [ 29_joint]\n\
            13_component_output(4 0) -> [ 22_output(1)]\n\
            14_component_input(5 0) -> [ 24_joint]\n\
            15_component_input(5 1) -> [ 26_joint]\n\
            16_component_input(5 2) -> [ 29_joint]\n\
            17_component_output(5 0) -> [ 23_output(2)]\n\
            18_input(0) -> [ 24_joint]\n\
            19_input(1) -> [ 26_joint]\n\
            20_input(2) -> [ 28_joint]\n\
            21_output(0) -> [ 9_component_output(3 0)]\n\
            22_output(1) -> [ 13_component_output(4 0)]\n\
            23_output(2) -> [ 17_component_output(5 0)]\n\
            24_joint -> [ 0_component_input(0 0), 14_component_input(5 0), 18_input(0)]\n\
            25_joint -> [ 10_component_input(4 0), 1_component_output(0 0), 6_component_input(3 0)]\n\
            26_joint -> [ 15_component_input(5 1), 19_input(1), 2_component_input(1 0)]\n\
            27_joint -> [ 11_component_input(4 1), 3_component_output(1 0), 7_component_input(3 1)]\n\
            28_joint -> [ 20_input(2), 29_joint, 4_component_input(2 0)]\n\
            29_joint -> [ 12_component_input(4 2), 16_component_input(5 2), 28_joint]\n\
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
        let error = parse(test_circuit, &HashMap::new()).unwrap_err();
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
