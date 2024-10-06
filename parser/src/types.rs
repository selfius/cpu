use std::fmt::{Display, Error, Formatter};
use std::ops::Range;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedSymbol(Position),
    Looping,
    UnexpectedState {
        position: Position,
        message: &'static str,
    },
    EndOfInput,
    InternalStateError,
}

#[derive(Debug)]
pub enum ParsingMode {
    Wire,
    Box,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Wire {
        start: Position,
        end: Position,
    },
    Box {
        top_left: Position,
        bottom_right: Position,
        inputs: Vec<Position>,
        outputs: Vec<Position>,
    },
    Text {
        line: usize,
        position: Range<usize>,
        value: String,
    },
    Input {
        position: Position,
    },
    Output {
        position: Position,
    },
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Debug)]
pub struct Symbol {
    pub position: Position,
    pub character: char,
    pub direction: &'static Direction,
    pub mode: ParsingMode,
}

impl Symbol {
    pub fn new(
        position: Position,
        character: char,
        direction: &'static Direction,
        mode: ParsingMode,
    ) -> Symbol {
        Symbol {
            direction,
            position,
            character,
            mode,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
