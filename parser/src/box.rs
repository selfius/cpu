use crate::structural_scan::ScannerResult;
use crate::types::{Direction, Node, ParseError, ParsingMode, Position, Symbol};
use std::collections::HashSet;

pub fn scan_box(
    input: &[&str],
    symbol: Symbol,
    context: &mut BoxParsingContext,
) -> Result<ScannerResult, ParseError> {
    if let '┏' | '┗' | '┓' | '┛' = symbol.character {
        if context.corners.contains(&symbol.character) {
            return Err(ParseError::UnexpectedSymbol(symbol.position));
        }
        context.corners.insert(symbol.character);
    }

    update_context(context, &symbol);

    let next_direction = calculate_next_direction(&symbol)?;

    let next_position = next_direction.move_cursor(symbol.position.clone());
    if next_position == context.starting_position {
        return Ok(ScannerResult {
            node: Some(Node::Box {
                top_left: context.top_left.take().ok_or(ParseError::UnexpectedState {
                    position: symbol.position.clone(),
                    message: "at this point we should always know where the top left corner is",
                })?,
                bottom_right: context
                    .bottom_right
                    .take()
                    .ok_or(ParseError::UnexpectedState {
                        position: symbol.position.clone(),
                        message:
                            "at this point we should always know where the bottom right corner is",
                    })?,
                inputs: std::mem::take(&mut context.inputs),
                outputs: std::mem::take(&mut context.outputs),
            }),
            parse_now: vec![],
            parse_later: vec![],
        });
    }

    let next_char = input[next_position.line]
        .chars()
        .nth(next_position.column)
        .ok_or(ParseError::EndOfInput)?;
    let mut parse_later = vec![];
    match next_char {
        '┠' => {
            parse_later.push(Symbol::new(
                next_position.clone(),
                '─',
                &Direction::Right,
                ParsingMode::Wire,
            ));
        }
        '┨' => {
            parse_later.push(Symbol::new(
                next_position.clone(),
                '─',
                &Direction::Left,
                ParsingMode::Wire,
            ));
        }
        _ => (),
    }

    let next_box_symbol = Symbol::new(next_position, next_char, next_direction, ParsingMode::Box);
    Ok(ScannerResult {
        node: None,
        parse_now: vec![next_box_symbol],
        parse_later,
    })
}

fn update_context(context: &mut BoxParsingContext, symbol: &Symbol) {
    match symbol.character {
        '┏' => context.top_left = Some(symbol.position.clone()),
        '┛' => context.bottom_right = Some(symbol.position.clone()),
        '┨' => context.inputs.push(symbol.position.clone()),
        '┠' => context.outputs.push(symbol.position.clone()),
        _ => (),
    }
}

fn calculate_next_direction(symbol: &Symbol) -> Result<&'static Direction, ParseError> {
    Ok(match (symbol.character, symbol.direction) {
        ('━', Direction::Left | Direction::Right)
        | ('┃', Direction::Up | Direction::Down)
        | ('┨', Direction::Up | Direction::Down)
        | ('┠', Direction::Up | Direction::Down) => symbol.direction,
        ('┛', Direction::Down) => &Direction::Left,
        ('┛', Direction::Right) => &Direction::Up,

        ('┗', Direction::Down) => &Direction::Right,
        ('┗', Direction::Left) => &Direction::Up,

        ('┏', Direction::Left) => &Direction::Down,
        ('┏', Direction::Up) => &Direction::Right,

        ('┓', Direction::Right) => &Direction::Down,
        ('┓', Direction::Up) => &Direction::Left,
        _ => return Err(ParseError::UnexpectedSymbol(symbol.position.clone())),
    })
}

pub struct BoxParsingContext {
    starting_position: Position,
    corners: HashSet<char>,
    top_left: Option<Position>,
    bottom_right: Option<Position>,
    inputs: Vec<Position>,
    outputs: Vec<Position>,
}

impl BoxParsingContext {
    pub fn new(starting_position: &Position) -> BoxParsingContext {
        BoxParsingContext {
            starting_position: starting_position.clone(),
            corners: HashSet::new(),
            top_left: None,
            bottom_right: None,
            inputs: vec![],
            outputs: vec![],
        }
    }
}
