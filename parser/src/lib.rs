use std::collections::{HashSet, VecDeque};

mod types;
use types::{Direction, Node, Symbol, ParseError, Position, ParsingMode};

mod wires;
use wires::scan_for_wire_end;

pub fn parse(source: &str) -> Result<Vec<Node>, ParseError> {
    // convert string to alighned 2d array
    let lines: Vec<_> = source.lines().collect();

    // find inputs as dangling -.*
    let dangling_inputs = find_dangling_inputs(&lines);

    // put them in a stack or a queue and start untangling according to rules
    let mut symbols: VecDeque<Symbol> = VecDeque::new();
    dangling_inputs.into_iter().for_each(|input| {
        symbols.push_back(Symbol::new(input, '─', Direction::Right, ParsingMode::Wire))
    });
    scan(&lines, symbols)
}



fn scan(input: &[&str], mut to_look_at: VecDeque<Symbol>) -> Result<Vec<Node>, ParseError> {
    let mut components = vec![];
    let mut debug_num = 0;
    let mut new_component = true;
    let mut wire_start = Position::new(0, 0);
    while let Some(symbol) = to_look_at.pop_front() {
        if new_component {
            wire_start = symbol.position.clone();
        }
        new_component = false;
        debug_num += 1;
        if debug_num > 10000 {
            return Err(ParseError::Looping);
        }

        if let Some(node) = match symbol.mode {
            ParsingMode::Wire => scan_for_wire_end(input, symbol, &mut to_look_at, &wire_start),
            ParsingMode::Box => scan_box(input, symbol, &mut to_look_at),
        }? {
            new_component = true;
            components.push(node);
        }
    }
    Ok(components)
}

fn scan_box(
    _input: &[&str],
    symbol: Symbol,
    _to_look_at: &mut VecDeque<Symbol>,
) -> Result<Option<Node>, ParseError> {
    println!("Found Box pin at {}", symbol.position);
    unimplemented!();
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
    #[should_panic]
    fn finds_boxes() {
        let test_circuit = "
                 ┌───┐
              ─┬─┤   ├─────
               │ └───┘     
               │   
               └────  
    ";
        match parse(test_circuit) {
            Ok(wires) => {
                assert!(wires.contains(&Node::Wire {
                    start: Position::new(2, 14),
                    end: Position::new(2, 17)
                }));
            }

            _ => panic!("unexpected parse error"),
        }
    }
}
