use crate::BitState;

pub fn nand(input: &[BitState], output: &mut [BitState]) {
    assert!(input.len() == 2, "NAND gate must have exactly two inputs");
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
        let mut output = vec![BitState::Undefined];
        nand(&[BitState::On, BitState::Off], &mut output);
        assert_eq!(output, [BitState::On]);

        nand(&[BitState::On, BitState::On], &mut output);
        assert_eq!(output, [BitState::Off]);
    }
}
