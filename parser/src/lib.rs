use std::collections::HashSet;

pub fn parse(source: &str) {
    // convert string to alighned 2d array
    let lines: Vec<_> = source.lines().skip_while(|line| line.is_empty()).collect();

    // find inputs as dangling -.*
    for (num, input) in find_dangling_inputs(&lines).iter().enumerate() {
        println!("input #{num} [{} {}]", input.line, input.column);
    }

    // put them in a stack or a queue and start untangling according to rules
    // rule!(Symbol.Wire(Right) = Symbol.Wire(Right) + '─'
    //        | Symbol.Wire(Down) + '└');
    //
    // rule!(Symbol.Box = Symbol.Wire(Horizontal) + '┤');
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

#[derive(Debug)]
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
    fn it_works() {
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
}
