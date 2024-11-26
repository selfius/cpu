use crate::{BitState, DigitalComponent};
use std::rc::Rc;

pub fn new() -> DigitalComponent {
    named("")
}

pub fn named(name: &str) -> DigitalComponent {
    DigitalComponent::named(2, 1, Rc::new(nand), name)
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
        let mut nand_component = DigitalComponent::new(2, 1, Rc::new(nand));
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
