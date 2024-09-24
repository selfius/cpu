use std::collections::{HashSet, VecDeque};
use std::fmt::{Display, Error, Formatter};

pub fn parse(source: &str) -> Result<Vec<Wire>, ParseError> {
    // convert string to alighned 2d array
    let lines: Vec<_> = source.lines().collect();

    // find inputs as dangling -.*
    let dangling_inputs = find_dangling_inputs(&lines);
    for (num, input) in dangling_inputs.iter().enumerate() {
        println!("input #{num} [{} {}]", input.line, input.column);
    }

    // put them in a stack or a queue and start untangling according to rules
    let mut symbols: VecDeque<Symbol> = VecDeque::new();
    dangling_inputs
        .into_iter()
        .for_each(|input| symbols.push_back(Symbol::new(input, '─', Direction::Right)));
    scan(&lines, symbols)
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedSymbol,
}

#[derive(Debug, PartialEq)]
pub struct Wire {
    start: Position,
    end: Position,
}

fn scan(input: &[&str], mut to_look_at: VecDeque<Symbol>) -> Result<Vec<Wire>, ParseError> {
    let mut found_wires = vec![];
    let mut debug_num = 0;
    let mut new_component = true;
    let mut wire_start = Position::new(0, 0);
    while let Some(symbol) = to_look_at.pop_front() {
        println!("#{} : {}", debug_num, symbol.character);
        if new_component {
            wire_start = symbol.position.clone();
        }
        new_component = false;
        debug_num += 1;
        if debug_num > 1000 {
            panic!("we're doing something wrong");
        }
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

        match next_char {
            Some('┬') => {
                found_wires.push(Wire {
                    start: wire_start.clone(),
                    end: next_position.clone(),
                });
                to_look_at.push_front(Symbol::new(next_position.clone(), '─', symbol.direction));
                to_look_at.push_front(Symbol::new(next_position, '│', Direction::Down));

                new_component = true;
            }
            Some(' ') | None => {
                found_wires.push(Wire {
                    start: wire_start.clone(),
                    end: symbol.position.clone(),
                });
                new_component = true;
            }
            Some(character) => {
                to_look_at.push_front(Symbol::new(next_position, character, next_direction));
            }
        };
    }
    Ok(found_wires)
}

#[derive(Clone, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn move_cursor(&self, Position { line, column }: Position) -> Position {
        match self {
            Direction::Up => Position::new(line - 1, column),
            Direction::Down => Position::new(line + 1, column),
            Direction::Right => Position::new(line, column + 1),
            Direction::Left => Position::new(line, column - 1),
        }
    }
}

struct Symbol {
    direction: Direction,
    position: Position,
    character: char,
}

impl Symbol {
    fn new(position: Position, character: char, direction: Direction) -> Symbol {
        Symbol {
            direction,
            position,
            character,
        }
    }
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

#[derive(Clone, Debug, PartialEq)]
struct Position {
    line: usize,
    column: usize,
}

impl Position {
    fn new(line: usize, column: usize) -> Position {
        Position { line, column }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}:{}", self.line, self.column)
    }
}

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
                assert!(wires.contains(&Wire {
                    start: Position::new(2, 14),
                    end: Position::new(2, 17)
                }));
                assert!(wires.contains(&Wire {
                    start: Position::new(2, 17),
                    end: Position::new(5, 17)
                }));
                assert!(wires.contains(&Wire {
                    start: Position::new(2, 17),
                    end: Position::new(5, 27)
                }));
                assert!(wires.contains(&Wire {
                    start: Position::new(4, 15),
                    end: Position::new(4, 25)
                }));
            }

            _ => panic!("unexpected parse error"),
        }
    }
}
