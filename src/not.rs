use super::composite_component_logic;
use crate::digital_component::DigitalComponent;
use crate::nand::Nand;
use crate::{BitState, ComponentGraph};

use std::collections::HashMap;

pub fn not(name: &str) -> DigitalComponent {
    let components = vec![Nand::new().dc()];
    let cg = ComponentGraph {
        components,
        wiring: HashMap::new(),
    };
    DigitalComponent::named(
        1,
        1,
        Box::new(composite_component_logic(
            cg,
            Box::new(map_inputs),
            Box::new(map_outputs),
        )),
        name,
    )
}

fn map_inputs(inputs: &Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    components[0].set_input(0, &inputs[0]);
    components[0].set_input(1, &inputs[0]);
}

fn map_outputs(outputs: &mut Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    outputs[0] = components[0].get_output(0).clone();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_gate() {
        let mut not_gate = not("");
        not_gate.set_input(0, &BitState::On);
        not_gate.resolve();
        assert_eq!(not_gate.get_output(0), &BitState::Off);

        not_gate.set_input(0, &BitState::Off);
        assert_eq!(not_gate.resolve(), true);
        assert_eq!(not_gate.get_output(0), &BitState::On);
    }
}
