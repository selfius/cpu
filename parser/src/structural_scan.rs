use std::collections::{HashSet, VecDeque};

use crate::types::{Direction, Node, ParseError, ParsingMode, Position, Symbol};

use crate::r#box::{scan_box, BoxParsingContext};
use crate::wires::scan_for_wire_end;

pub fn structural_scan(
    input: &[&str],
    mut to_look_at: VecDeque<Symbol>,
) -> Result<Vec<Node>, ParseError> {
    let mut components = vec![];
    let mut debug_num = 0;
    let mut new_component = true;
    let mut wire_start = Position::new(0, 0);
    let mut box_parsing_context = BoxParsingContext::new(&wire_start);
    let mut positions_visited_vertically = HashSet::new();
    let mut positions_visited_horizontally = HashSet::new();
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

        new_component = false;

        debug_num += 1;
        if debug_num > 10000 {
            println!("{:?} {:?}", to_look_at, symbol);
            return Err(ParseError::Looping);
        }

        match symbol.direction {
            &Direction::Up | &Direction::Down => {
                positions_visited_vertically.insert(symbol.position.clone())
            }
            &Direction::Left | &Direction::Right => {
                positions_visited_horizontally.insert(symbol.position.clone())
            }
        };

        let scanner_result = match symbol.mode {
            ParsingMode::Wire => scan_for_wire_end(input, symbol, &wire_start),
            ParsingMode::Box => scan_box(input, symbol, &mut box_parsing_context),
        }?;

        if let Some(node) = scanner_result.node {
            components.push(node);
            new_component = true;
        }

        for to_front in scanner_result.parse_now {
            if !(match to_front.direction {
                &Direction::Up | &Direction::Down => &positions_visited_vertically,
                &Direction::Left | &Direction::Right => &positions_visited_horizontally,
            })
            .contains(&to_front.position)
            {
                to_look_at.push_front(to_front);
            }
        }

        for to_back in scanner_result.parse_later {
            if !(match to_back.direction {
                &Direction::Up | &Direction::Down => &positions_visited_vertically,
                &Direction::Left | &Direction::Right => &positions_visited_horizontally,
            })
            .contains(&to_back.position)
            {
                to_look_at.push_back(to_back);
            }
        }
    }
    Ok(components)
}
