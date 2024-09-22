use std::collections::{HashSet, VecDeque};

pub fn parse(source: &str) {
    // convert string to alighned 2d array
    let lines: Vec<_> = source.lines().skip_while(|line| line.is_empty()).collect();

    // find inputs as dangling -.*
    let dangling_inputs = find_dangling_inputs(&lines);
    for (num, input) in dangling_inputs.iter().enumerate() {
        println!("input #{num} [{} {}]", input.line, input.column);
    }

    // put them in a stack or a queue and start untangling according to rules
    // rule!(Symbol.Wire(Right) = Symbol.Wire(Right) + '─'
    //        | Symbol.Wire(Down) + '└');
    //
    // rule!(Symbol.Box = Symbol.Wire(Horizontal) + '┤');
    let mut symbols: VecDeque<Symbol> = VecDeque::new();
    dangling_inputs
        .into_iter()
        .for_each(|input| symbols.push_back(Symbol::new(SymbolKind::Wire, input, '─')));
    scan(&lines, symbols);
}

fn scan(input: &[&str], mut to_look_at: VecDeque<Symbol>) {
    let mut current_direction = Direction::Right;
    let mut debug_num = 0;
    while let Some(symbol) = to_look_at.pop_front() {
        println!("#{} : {}", debug_num, symbol.character);
        debug_num += 1;
        //separate logic for when we split

        //when we're just keep chugging along
        let next_direction = match (symbol.character, current_direction.clone()) {
            ('─', Direction::Left | Direction::Right) | ('│', Direction::Up | Direction::Down) => {
                current_direction.clone()
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
            _ => panic!(
                "unexpected symbol {} at {}:{} while going {:?}",
                symbol.character, symbol.position.line, symbol.position.column, current_direction
            ),
        };

        let next_position = next_direction.move_cursor(symbol.position.clone());
        let next_char = input[next_position.line].chars().nth(next_position.column);

        match next_char {
            Some(' ') | None => println!(
                "reached the end of this wire at {}:{}",
                symbol.position.line, symbol.position.column
            ),
            Some(symbol) => {
                to_look_at.push_front(Symbol::new(SymbolKind::Wire, next_position, symbol))
            }
        };

        current_direction = next_direction;
    }
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

enum SymbolKind {
    Wire,
}

struct Symbol {
    kind: SymbolKind,
    position: Position,
    character: char,
}

impl Symbol {
    fn new(kind: SymbolKind, position: Position, character: char) -> Symbol {
        Symbol {
            kind,
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

#[derive(Clone)]
struct Position {
    line: usize,
    column: usize,
}

impl Position {
    fn new(line: usize, column: usize) -> Position {
        Position { line, column }
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

        parse(test_circuit);
    }

    #[test]
    fn simple_wiring() {
        let test_circuit = "
                       ┌───┐
              ──────┐  │   │
                    └──┼───┘
                       │    
                       └────
    ";
        parse(test_circuit);
    }
}
