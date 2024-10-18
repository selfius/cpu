use std::collections::VecDeque;

mod types;
use types::{Direction, Node, ParseError, ParsingMode, Symbol};

mod wires;

mod r#box;

mod structural_scan;
use structural_scan::{find_dangling_wires, structural_scan};

mod node_graph;
use node_graph::build_node_graph;

use digital_component::Graph;

pub fn parse(source: &str) -> Result<Graph, ParseError> {
    // scan what take break into what would be equivalent of a 2D token
    let result = scan(source)?;

    // no build an graph where wires from previous stage are edges and the rest is nodes
    build_node_graph(result)
}

fn scan(source: &str) -> Result<Vec<Node>, ParseError> {
    // convert string to alighned 2d array
    let lines: Vec<_> = source.lines().collect();
    // find inputs as dangling -.*
    let (dangling_inputs, dangling_outputs) = find_dangling_wires(&lines);

    // put them in a stack or a queue and start untangling according to rules
    let mut symbols: VecDeque<Symbol> = VecDeque::new();
    dangling_inputs.iter().for_each(|input_position| {
        symbols.push_back(Symbol::new(
            input_position.clone(),
            '─',
            &Direction::Right,
            ParsingMode::Wire,
        ))
    });
    let mut nodes = scan_for_text_tokens(&lines);
    nodes.append(&mut structural_scan(&lines, symbols)?);
    nodes.append(
        &mut dangling_inputs
            .into_iter()
            .map(|position| Node::Input { position })
            .collect(),
    );
    nodes.append(
        &mut dangling_outputs
            .into_iter()
            .map(|position| Node::Output { position })
            .collect(),
    );
    Ok(nodes)
}

fn scan_for_text_tokens(input: &[&str]) -> Vec<Node> {
    let mut nodes = vec![];
    let mut current_state = TextTokenFSMState::Junk;
    let mut current_token = String::new();
    let mut token_start = 0;
    for (line_num, line) in input.iter().enumerate() {
        for (char_num, c) in line.chars().chain([' ']).enumerate() {
            match (c, &current_state) {
                ('a'..='z' | 'A'..='Z', TextTokenFSMState::Junk) => {
                    current_state = TextTokenFSMState::Text;
                    current_token.push(c);
                    token_start = char_num;
                }
                ('a'..='z' | 'A'..='Z' | '0'..='9' | '_', TextTokenFSMState::Text) => {
                    current_token.push(c);
                }
                (_, TextTokenFSMState::Text) => {
                    nodes.push(Node::Text {
                        line: line_num,
                        position: token_start..char_num,
                        value: current_token,
                    });
                    current_state = TextTokenFSMState::Junk;
                    current_token = String::new();
                }
                _ => {}
            }
        }
    }
    nodes
}

enum TextTokenFSMState {
    Junk,
    Text,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertor::*;
    use types::*;

    #[test]
    fn aspiration() {
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
    fn simple_wiring() {
        let test_circuit = "
                       ┌───┐
              ───┬──┐  │   │
                 │  └──┼───┘
               ──┼─────┼──  
                 │     └────
    ";
        let wires = scan(test_circuit).unwrap();
        assert_that!(wires
            .into_iter()
            .filter(|node| matches!(node, Node::Wire { .. }))
            .collect::<Vec<_>>())
        .contains_exactly(vec![
            Node::Wire {
                start: Position::new(2, 17),
                end: Position::new(5, 17),
            },
            Node::Wire {
                start: Position::new(2, 14),
                end: Position::new(2, 17),
            },
            Node::Wire {
                start: Position::new(2, 17),
                end: Position::new(5, 27),
            },
            Node::Wire {
                start: Position::new(4, 15),
                end: Position::new(4, 25),
            },
        ]);
    }

    #[test]
    fn find_inputs_and_outputs() {
        let test_circuit = "
                       ┌───┐
              ───┬──┐  │   │
                 │  └──┼───┘
               ──┼─────┼──  
                 │     └────
    ";
        let wires = scan(test_circuit).unwrap();
        assert_that!(wires
            .into_iter()
            .filter(|node| matches!(node, Node::Input { .. } | Node::Output { .. }))
            .collect::<Vec<_>>())
        .contains_exactly(vec![
            Node::Input {
                position: Position::new(2, 14),
            },
            Node::Input {
                position: Position::new(4, 15),
            },
            Node::Output {
                position: Position::new(4, 25),
            },
            Node::Output {
                position: Position::new(5, 27),
            },
        ]);
    }

    #[test]
    fn finds_boxes() {
        let test_circuit = "
                 ┏━━━┓
              ─┬─┨   ┠─┬──
               │ ┃   ┠─┼─┐ 
               │ ┗━━━┛ │ │
               ├──┬────┘ │
               └──┴──────┘
    ";
        assert_that!(scan(test_circuit).unwrap()).contains(&Node::Box {
            top_left: Position::new(1, 17),
            bottom_right: Position::new(4, 21),
            inputs: vec![Position::new(2, 17)],
            outputs: vec![Position::new(3, 21), Position::new(2, 21)],
        });
    }

    #[test]
    fn finds_text_tokens() {
        let test_circuit = "
                 ┏━━━━━━━━━┓
              ───┨ token1  ┠─
                 ┃   token2┃
                 ┗━━━━━━━━━┛
               tok_en3;4token4
token5              %$#
    ";
        let nodes = scan(test_circuit).unwrap();
        assert_that!(nodes).contains(&Node::Box {
            top_left: Position::new(1, 17),
            bottom_right: Position::new(4, 27),
            inputs: vec![Position::new(2, 17)],
            outputs: vec![Position::new(2, 27)],
        });
        assert_that!(nodes).contains(&Node::Text {
            line: 2,
            position: 19..25,
            value: String::from("token1"),
        });
        assert_that!(nodes).contains(&Node::Text {
            line: 3,
            position: 21..27,
            value: String::from("token2"),
        });
        assert_that!(nodes).contains(&Node::Text {
            line: 5,
            position: 15..22,
            value: String::from("tok_en3"),
        });
        assert_that!(nodes).contains(&Node::Text {
            line: 5,
            position: 24..30,
            value: String::from("token4"),
        });
        assert_that!(nodes).contains(&Node::Text {
            line: 6,
            position: 0..6,
            value: String::from("token5"),
        });
    }
}
