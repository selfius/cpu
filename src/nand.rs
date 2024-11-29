use crate::BitState;
use std::cell::RefCell;

pub fn nand(input: &[BitState], output: &RefCell<Vec<BitState>>) {
    assert!(input.len() == 2, "NAND gate must have exactly two inputs");
    let mut output = output.borrow_mut();
    assert!(output.len() == 1, "NAND gate must have exactly one output");
    output[0] = match (&input[0], &input[1]) {
        (BitState::On, BitState::On) => BitState::Off,
        (BitState::Undefined, BitState::Undefined) => BitState::Undefined,
        _ => BitState::On,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nand_gate() {
        let output = RefCell::new(vec![BitState::Undefined]);
        nand(&[BitState::On, BitState::Off], &output);
        assert_eq!(output.borrow()[..], [BitState::On]);

        nand(&[BitState::On, BitState::On], &output);
        assert_eq!(output.borrow()[..], [BitState::Off]);
    }
}
