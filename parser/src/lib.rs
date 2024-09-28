use std::collections::{HashSet, VecDeque};

mod types;
use types::{Direction, Node, ParseError, ParsingMode, Position, Symbol};

mod wires;
use wires::scan_for_wire_end;

mod r#box;
use r#box::{scan_box, BoxParsingContext};

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
    scan(&lines, symbols)
}

fn scan(input: &[&str], mut to_look_at: VecDeque<Symbol>) -> Result<Vec<Node>, ParseError> {
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

const WIRE_SYMBOLS: &str = "─│┬┴┘┐┌└┼└┘";
const BOX_SYMBOLS: &str = "─│┐┌└┘├┤";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn aspiration() {
        let test_circuit = "
                 ┌───┐
              ─┬─┤   ├─────┐
               │ └───┘     │
               │   ┌───┐   │
              ─┼─┬─┤   ├───┼─┐
               │ │ └───┘   │ │
               │ │   ┌───┐ │ │
              ─┼─┼─┬─┤   ├─┼─┼─┐
               │ │ │ └───┘ │ │ │ ┌─────┐
               │ │ │       ├─┼─┼─┤     │
               │ │ │       │ ├─┼─┤     ├─
               │ │ │       │ │ └─┤     │
               │ │ │       │ │   └─────┘
               │ │ │       │ │   ┌─────┐
               │ │ │       └─┼───┤     │
               │ │ │         └───┤     ├─
               │ │ ├─────────────┤     │
               │ │ │             └─────┘
               │ │ │             ┌─────┐
               └─┼─┼─────────────┤     │
                 └─┼─────────────┤     ├─
                   └─────────────┤     │
                                 └─────┘
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
                 ┌───┐
              ─┬─┤   ├─┬──
               │ │   ├─┼─┐ 
               │ └───┘ │ │
               └──┬────┘ │
                  └──────┘
    ";
        match parse(test_circuit) {
            Ok(components) => {
                assert!(components.contains(&Node::Box {
                    top_left: Position::new(1, 17),
                    bottom_right: Position::new(4, 21)
                }));
                assert_eq!(components.len(), 8);
            }
            Err(error) => panic!("{:?}", error),
        }
    }
}
