use std::fmt::{Display, Error, Formatter};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedSymbol,
}

pub enum ParsingMode {
    Wire,
    Box,
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Wire {
        start: Position,
        end: Position,
    },
    Box {
        top_left: Position,
        bottom_right: Position,
    },
}

#[derive(Clone, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn move_cursor(&self, Position { line, column }: Position) -> Position {
        match self {
            Direction::Up => Position::new(line - 1, column),
            Direction::Down => Position::new(line + 1, column),
            Direction::Right => Position::new(line, column + 1),
            Direction::Left => Position::new(line, column - 1),
        }
    }
}

pub struct Symbol {
    pub position: Position,
    pub character: char,
    pub direction: Direction,
    pub mode: ParsingMode,
}

impl Symbol {
    pub fn new(position: Position, character: char, direction: Direction, mode: ParsingMode) -> Symbol {
        Symbol {
            direction,
            position,
            character,
            mode,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Position {
        Position { line, column }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}:{}", self.line, self.column)
    }
}
