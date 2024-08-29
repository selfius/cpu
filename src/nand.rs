use crate::{BitState, DigitalComponent};

pub struct Nand {
    dc: DigitalComponent,
}

impl Nand {
    pub fn new() -> Nand {
        Nand::named("")
    }

    pub fn named(name: &str) -> Nand {
        Nand {
            dc: DigitalComponent::named(2, 1, Box::new(nand), name),
        }
    }

    pub fn dc(self) -> DigitalComponent {
        self.dc
    }
}

fn nand(input: &Vec<BitState>, output: &mut Vec<BitState>) -> bool {
    assert!(input.len() == 2, "NAND gate must have exactly two inputs");
    assert!(output.len() == 1, "NAND gate must have exactly one ouput");
    let previous_value = output[0].clone();
    output[0] = match (&input[0], &input[1]) {
        (BitState::On, BitState::On) => BitState::Off,
        (BitState::Undefined, BitState::Undefined) => BitState::Undefined,
        _ => BitState::On,
    };
    previous_value != output[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BitState, DigitalComponent};

    #[test]
    fn nand_gate() {
        let mut nand_component = DigitalComponent::new(2, 1, Box::new(nand));
        assert_eq!(nand_component.get_output(0), &BitState::Undefined);

        nand_component.set_input(0, &BitState::On);
        nand_component.set_input(1, &BitState::Off);
        assert_eq!(nand_component.resolve(), true);
        assert_eq!(nand_component.get_output(0), &BitState::On);

        nand_component.set_input(1, &BitState::On);
        assert_eq!(nand_component.resolve(), true);
        assert_eq!(nand_component.get_output(0), &BitState::Off);
    }
}
