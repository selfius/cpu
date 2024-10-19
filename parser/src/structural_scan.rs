use std::collections::{HashSet, VecDeque};

use crate::types::{Node, ParseError, ParsingMode, Position, Symbol};

use crate::r#box::{scan_box, BoxParsingContext};
use crate::wires::{scan_for_wire_end, WIRE_SYMBOLS};

pub fn structural_scan(
    input: &[&str],
    mut to_look_at: VecDeque<Symbol>,
) -> Result<Vec<Node>, ParseError> {
    let mut components = vec![];
    let mut debug_num = 0;
    let mut new_component = true;
    let mut wire_start = Position::new(0, 0);
    let mut box_parsing_context = BoxParsingContext::new(&wire_start);
    let mut visited: HashSet<Position> = HashSet::default();
    while let Some(symbol) = to_look_at.pop_front() {
        if new_component {
            match symbol.mode {
                ParsingMode::Wire => {
                    wire_start = symbol.position.clone();
                }
                ParsingMode::Box => {
                    box_parsing_context = BoxParsingContext::new(&symbol.position);
                }
            }
        }

        debug_num += 1;
        if debug_num > 10000 {
            println!("{:?} {:?}", to_look_at, symbol);
            return Err(ParseError::Looping);
        }

        new_component = false;

        let scanner_result = match symbol.mode {
            ParsingMode::Wire => scan_for_wire_end(input, symbol, &wire_start, &mut visited),
            ParsingMode::Box => scan_box(input, symbol, &mut box_parsing_context, &mut visited),
        }?;
        if scanner_result.is_empty() {
            new_component = true;
        }

        if let Some(node) = scanner_result.node {
            components.push(node);
            new_component = true;
        }

        for to_front in scanner_result.parse_now {
            to_look_at.push_front(to_front);
        }

        for to_back in scanner_result.parse_later {
            to_look_at.push_back(to_back);
        }
    }
    println!("visited: {visited:?}");
    Ok(components)
}

pub fn find_dangling_wires(input: &[&str]) -> (Vec<Position>, Vec<Position>) {
    let mut dangling_inputs = vec![];
    let mut dangling_outputs = vec![];
    let struct_symbol_set: HashSet<_> = WIRE_SYMBOLS.chars().chain(BOX_SYMBOLS.chars()).collect();
    for (line_num, line) in input.iter().enumerate() {
        let mut prev_symbol: Option<char> = None;
        for (col_num, symbol) in line.chars().chain([' ']).enumerate() {
            match (prev_symbol, symbol) {
                (Some('─'), junk) if !struct_symbol_set.contains(&junk) => {
                    dangling_outputs.push(Position::new(line_num, col_num - 1));
                }
                (Some(junk), '─') if !struct_symbol_set.contains(&junk) => {
                    dangling_inputs.push(Position::new(line_num, col_num));
                }
                (None, '─') => {
                    dangling_inputs.push(Position::new(line_num, col_num));
                }
                _ => (),
            }
            prev_symbol = Some(symbol);
        }
    }
    (dangling_inputs, dangling_outputs)
}

const BOX_SYMBOLS: &str = "━┃┓┏┗┛┠┨";

#[derive(Default)]
pub struct ScannerResult {
    pub node: Option<Node>,
    pub parse_now: Vec<Symbol>,
    pub parse_later: Vec<Symbol>,
}

impl ScannerResult {
    pub fn is_empty(&self) -> bool {
        self.node.is_none() && self.parse_now.is_empty() && self.parse_later.is_empty()
    }
}
