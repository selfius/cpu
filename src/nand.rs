use crate::{BitState, DigitalComponent};

pub fn new<'a>() -> DigitalComponent<'a> {
    named("")
}

pub fn named<'a>(name: &str) -> DigitalComponent<'a> {
    DigitalComponent::named(2, 1, &nand, name)
}

fn nand(input: &[BitState]) -> Vec<BitState> {
    assert!(input.len() == 2, "NAND gate must have exactly two inputs");
    let mut output = vec![BitState::Undefined];
    output[0] = match (&input[0], &input[1]) {
        (BitState::On, BitState::On) => BitState::Off,
        (BitState::Undefined, BitState::Undefined) => BitState::Undefined,
        _ => BitState::On,
    };
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BitState, DigitalComponent};

    #[test]
    fn nand_gate() {
        let mut nand_component = DigitalComponent::new(2, 1, &nand);
        assert_eq!(nand_component.get_output(0), &BitState::Undefined);

        nand_component.set_input(0, &BitState::On);
        nand_component.set_input(1, &BitState::Off);
        assert!(nand_component.resolve());
        assert_eq!(nand_component.get_output(0), &BitState::On);

        nand_component.set_input(1, &BitState::On);
        assert!(nand_component.resolve());
        assert_eq!(nand_component.get_output(0), &BitState::Off);
    }
}
