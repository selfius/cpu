use crate::structural_scan::ScannerResult;
use crate::types::{Direction, Node, ParseError, ParsingMode, Position, Symbol};
use std::collections::HashSet;
pub const WIRE_JOINT: &str = "┬┴├┤";

pub const WIRE_SYMBOLS: &str = "─│┬┴┘┐┌└┼└┘├┤";

pub fn scan_for_wire_end(
    input: &[&str],
    symbol: Symbol,
    wire_start: &Position,
    visited: &mut HashSet<Position>,
) -> Result<ScannerResult, ParseError> {
    Ok(match symbol.character {
        split if WIRE_JOINT.contains(split) => {
            let parse_next = [
                ('─', &Direction::Left),
                ('─', &Direction::Right),
                ('│', &Direction::Down),
                ('│', &Direction::Up),
            ]
            .iter()
            .filter(|(_, dir)| match split {
                '┬' => *dir != &Direction::Up,
                '┴' => *dir != &Direction::Down,
                '├' => *dir != &Direction::Left,
                '┤' => *dir != &Direction::Right,
                _ => panic!("this can not be None under any circumstances"),
            })
            .filter(|(_, dir)| *dir != &symbol.direction.opposite())
            .map(|(c, dir)| Symbol::new(symbol.position.clone(), *c, dir, ParsingMode::Wire))
            .collect();

            ScannerResult {
                node: Some(Node::Wire {
                    start: wire_start.clone(),
                    end: symbol.position,
                }),
                parse_now: parse_next,
                parse_later: vec![],
            }
        }
        box_pin if box_pin == '┨' || box_pin == '┠' => ScannerResult {
            node: Some(Node::Wire {
                start: wire_start.clone(),
                end: symbol.position.clone(),
            }),
            parse_now: vec![Symbol::new(
                symbol.position.clone(),
                box_pin,
                &Direction::Up,
                ParsingMode::Box,
            )],
            parse_later: vec![],
        },
        _ => {
            if let Some(underlying_char) = input[symbol.position.line]
                .chars()
                .nth(symbol.position.column)
            {
                if !WIRE_JOINT.contains(underlying_char) && '┼' != underlying_char {
                    if visited.contains(&symbol.position) {
                        return Ok(ScannerResult::default());
                    } else {
                        visited.insert(symbol.position.clone());
                    }
                }
                follow_wire(&symbol, input, wire_start)?
            } else {
                panic!("Missing structural symbol at {:?}", symbol.position);
            }
        }
    })
}

fn follow_wire(
    symbol: &Symbol,
    input: &[&str],
    wire_start: &Position,
) -> Result<ScannerResult, ParseError> {
    let next_direction: &Direction = match (symbol.character, symbol.direction) {
        ('─', Direction::Left | Direction::Right) | ('│', Direction::Up | Direction::Down) => {
            symbol.direction
        }
        ('┼', dir) => dir,
        ('┘', Direction::Down) => &Direction::Left,
        ('┘', Direction::Right) => &Direction::Up,

        ('└', Direction::Down) => &Direction::Right,
        ('└', Direction::Left) => &Direction::Up,

        ('┌', Direction::Left) => &Direction::Down,
        ('┌', Direction::Up) => &Direction::Right,

        ('┐', Direction::Right) => &Direction::Down,
        ('┐', Direction::Up) => &Direction::Left,
        _ => return Err(ParseError::UnexpectedSymbol(symbol.position.clone())),
    };

    let next_position = next_direction.move_cursor(symbol.position.clone());
    let next_char = input[next_position.line].chars().nth(next_position.column);
    Ok(match next_char {
        Some(character) if WIRE_SYMBOLS.contains(character) || "┨┠".contains(character) => {
            ScannerResult {
                node: None,
                parse_now: vec![Symbol::new(
                    next_position,
                    character,
                    next_direction,
                    ParsingMode::Wire,
                )],
                parse_later: vec![],
            }
        }
        _ => ScannerResult {
            node: Some(Node::Wire {
                start: wire_start.clone(),
                end: symbol.position.clone(),
            }),
            parse_now: vec![],
            parse_later: vec![],
        },
    })
}


#[cfg(test)]
mod tests {
    use crate::scan;
    use assertor::*;
    use crate::types::*;

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
    fn handle_wire_loops() {
        let test_circuit = "
                       ┌───┐
              ───┬──┐  │   │
                 │  └──┴───┘
    ";
        let wires = scan(test_circuit).unwrap();
        assert_that!(wires
            .into_iter()
            .filter(|node| matches!(node, Node::Wire { .. }))
            .collect::<Vec<_>>())
        .contains_exactly(vec![
            Node::Wire {
                start: Position::new(2, 14),
                end: Position::new(2, 17),
            },
            Node::Wire {
                start: Position::new(2, 17),
                end: Position::new(3, 17),
            },
            Node::Wire {
                start: Position::new(2, 17),
                end: Position::new(3, 23),
            },
            Node::Wire {
                start: Position::new(3, 23),
                end: Position::new(3, 23),
            },
        ]);
    }
}
