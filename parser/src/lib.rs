use std::collections::{HashSet, VecDeque};

mod types;
use types::{Direction, Node, ParseError, ParsingMode, Position, Symbol};

mod wires;

mod r#box;

mod structural_scan;
use structural_scan::structural_scan;

pub fn parse(source: &str) -> Result<Vec<Node>, ParseError> {
    // convert string to alighned 2d array
    let lines: Vec<_> = source.lines().collect();

    // find inputs as dangling -.*
    let dangling_inputs = find_dangling_inputs(&lines);

    // put them in a stack or a queue and start untangling according to rules
    let mut symbols: VecDeque<Symbol> = VecDeque::new();
    dangling_inputs.into_iter().for_each(|input| {
        symbols.push_back(Symbol::new(
            input,
            '─',
            &Direction::Right,
            ParsingMode::Wire,
        ))
    });
    let mut nodes = scan_for_text_tokens(&lines);
    nodes.append(&mut structural_scan(&lines, symbols)?);
    Ok(nodes)
}

struct ScannerResult {
    node: Option<Node>,
    parse_now: Vec<Symbol>,
    parse_later: Vec<Symbol>,
}

fn find_dangling_inputs(input: &[&str]) -> Vec<Position> {
    let mut dangling_inputs = vec![];
    let struct_symbol_set: HashSet<_> = WIRE_SYMBOLS.chars().chain(BOX_SYMBOLS.chars()).collect();
    for (line_num, line) in input.iter().enumerate() {
        let mut prev_symbol: Option<char> = None;
        for (col_num, symbol) in line.chars().enumerate() {
            if symbol == '─'
                && prev_symbol
                    .filter(|sym| struct_symbol_set.contains(sym))
                    .is_none()
            {
                //yeild line and column if it's a horizontal wire with nothing to it's left
                dangling_inputs.push(Position::new(line_num, col_num));
            }
            prev_symbol = Some(symbol);
        }
    }
    dangling_inputs
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
               (_ , TextTokenFSMState::Text) => {
                    nodes.push(Node::Text{
                        line: line_num,
                        position: token_start..char_num,
                        value: current_token,
                    });
                    current_state = TextTokenFSMState::Junk;
                    current_token = String::new();
               }
               _ => {
               }
           }
       }
    }
    nodes
}

enum TextTokenFSMState {
    Junk,
    Text,
}

const WIRE_SYMBOLS: &str = "─│┬┴┘┐┌└┼└┘";
const BOX_SYMBOLS: &str = "━┃┓┏┗┛┠┨";

#[cfg(test)]
mod tests {
    use super::*;
    use assertor::*;

    #[test]
    fn aspiration() {
        let test_circuit = "
                 ┏━━━┓                     
              ─┬─┨   ┠─────┐               
               │ ┗━━━┛     │               
               │   ┏━━━┓   │               
              ─┼─┬─┨   ┠───┼─┐             
               │ │ ┗━━━┛   │ │             
               │ │   ┏━━━┓ │ │             
              ─┼─┼─┬─┨   ┠─┼─┼─┐           
               │ │ │ ┗━━━┛ │ │ │ ┏━━━━━┓   
               │ │ │       ├─┼─┼─┨     ┃   
               │ │ │       │ ├─┼─┨     ┠─  
               │ │ │       │ │ └─┨     ┃   
               │ │ │       │ │   ┗━━━━━┛   
               │ │ │       │ │   ┏━━━━━┓   
               │ │ │       └─┼───┨     ┃   
               │ │ │         └───┨     ┠─  
               │ │ ├─────────────┨     ┃   
               │ │ │             ┗━━━━━┛   
               │ │ │             ┏━━━━━┓   
               └─┼─┼─────────────┨     ┃   
                 └─┼─────────────┨     ┠─  
                   └─────────────┨     ┃   
                                 ┗━━━━━┛   
    ";

        let _ = parse(test_circuit);
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
        match parse(test_circuit) {
            Ok(wires) => {
                assert!(wires.contains(&Node::Wire {
                    start: Position::new(2, 14),
                    end: Position::new(2, 17)
                }));
                assert!(wires.contains(&Node::Wire {
                    start: Position::new(2, 17),
                    end: Position::new(5, 17)
                }));
                assert!(wires.contains(&Node::Wire {
                    start: Position::new(2, 17),
                    end: Position::new(5, 27)
                }));
                assert!(wires.contains(&Node::Wire {
                    start: Position::new(4, 15),
                    end: Position::new(4, 25)
                }));
            }

            _ => panic!("unexpected parse error"),
        }
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
        assert_that!(parse(test_circuit).unwrap()).contains(&Node::Box {
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
        let nodes = parse(test_circuit).unwrap();
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
