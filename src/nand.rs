use crate::{
    BitState,
    BitState::{Off, On, Undefined},
};

pub fn nand(input: &[BitState], output: &mut [BitState]) {
    assert!(input.len() == 2, "NAND gate must have exactly two inputs");
    assert!(output.len() == 1, "NAND gate must have exactly one output");
    output[0] = match (&input[0], &input[1]) {
        (On, On) => Off,
        (Off, _) | (_, Off) => On,
        (Undefined, _) | (_, Undefined) => Undefined,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nand_gate() {
        let mut output = vec![BitState::Undefined];
        nand(&[BitState::On, BitState::Off], &mut output);
        assert_eq!(output, [BitState::On]);

        nand(&[BitState::On, BitState::On], &mut output);
        assert_eq!(output, [BitState::Off]);
    }
}
