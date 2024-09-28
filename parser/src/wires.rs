use super::ScannerResult;
use crate::types::{Direction, Node, ParseError, ParsingMode, Position, Symbol};

pub fn scan_for_wire_end(
    input: &[&str],
    symbol: Symbol,
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
        _ => return Err(ParseError::UnexpectedSymbol(symbol.position)),
    };

    let next_position = next_direction.move_cursor(symbol.position.clone());
    let next_char = input[next_position.line].chars().nth(next_position.column);

    Ok(match next_char {
        split @ Some('┬' | '┴' | '├' | '┤') => {
            let parse_next = [
                ('─', &Direction::Right),
                ('─', &Direction::Left),
                ('│', &Direction::Down),
                ('│', &Direction::Up),
            ]
            .iter()
            .filter(|(_, dir)| match split {
                Some('┬') => *dir != &Direction::Up,
                Some('┴') => *dir != &Direction::Down,
                Some('├') => *dir != &Direction::Left,
                Some('┤') => *dir != &Direction::Right,
                _ => panic!("this can not be None under any circumstances"),
            })
            .map(|(c, dir)| Symbol::new(next_position.clone(), *c, dir, ParsingMode::Wire))
            .collect();

            ScannerResult {
                node: Some(Node::Wire {
                    start: wire_start.clone(),
                    end: next_position,
                }),
                parse_now: parse_next,
                parse_later: vec![],
            }
        }
        box_pin @ Some('┨' | '┠') => ScannerResult {
            node: Some(Node::Wire {
                start: wire_start.clone(),
                end: next_position.clone(),
            }),
            parse_now: vec![Symbol::new(
                next_position,
                box_pin.unwrap(),
                &Direction::Up,
                ParsingMode::Box,
            )],
            parse_later: vec![],
        },
        Some(' ') | None => ScannerResult {
            node: Some(Node::Wire {
                start: wire_start.clone(),
                end: symbol.position.clone(),
            }),
            parse_now: vec![],
            parse_later: vec![],
        },
        Some(character) => ScannerResult {
            node: None,
            parse_now: vec![Symbol::new(
                next_position,
                character,
                next_direction,
                ParsingMode::Wire,
            )],
            parse_later: vec![],
        },
    })
}
