use std::collections::VecDeque;
use crate::types::{Direction, Node, Symbol, ParseError, Position, ParsingMode};

pub fn scan_for_wire_end(
    input: &[&str],
    symbol: Symbol,
    to_look_at: &mut VecDeque<Symbol>,
    wire_start: &Position,
) -> Result<Option<Node>, ParseError> {
    //when we're just keep chugging along
    let next_direction = match (symbol.character, symbol.direction.clone()) {
        ('─', Direction::Left | Direction::Right) | ('│', Direction::Up | Direction::Down) => {
            symbol.direction.clone()
        }
        ('┼', dir) => dir,
        ('┘', Direction::Down) => Direction::Left,
        ('┘', Direction::Right) => Direction::Up,

        ('└', Direction::Down) => Direction::Right,
        ('└', Direction::Left) => Direction::Up,

        ('┌', Direction::Left) => Direction::Down,
        ('┌', Direction::Up) => Direction::Right,

        ('┐', Direction::Right) => Direction::Down,
        ('┐', Direction::Up) => Direction::Left,
        _ => return Err(ParseError::UnexpectedSymbol),
    };

    let next_position = next_direction.move_cursor(symbol.position.clone());
    let next_char = input[next_position.line].chars().nth(next_position.column);

    Ok(match next_char {
        Some('┬') => {
            to_look_at.push_front(Symbol::new(
                next_position.clone(),
                '─',
                symbol.direction,
                ParsingMode::Wire,
            ));
            to_look_at.push_front(Symbol::new(
                next_position.clone(),
                '│',
                Direction::Down,
                ParsingMode::Wire,
            ));
            Some(Node::Wire {
                start: wire_start.clone(),
                end: next_position,
            })
        }
        box_pin @ Some('┤' | '├') => {
            to_look_at.push_front(Symbol::new(
                next_position.clone(),
                box_pin.unwrap(),
                symbol.direction,
                ParsingMode::Box,
            ));
            Some(Node::Wire {
                start: wire_start.clone(),
                end: next_position,
            })
        }
        Some(' ') | None => Some(Node::Wire {
            start: wire_start.clone(),
            end: symbol.position.clone(),
        }),
        Some(character) => {
            to_look_at.push_front(Symbol::new(
                next_position,
                character,
                next_direction,
                ParsingMode::Wire,
            ));
            None
        }
    })
}
