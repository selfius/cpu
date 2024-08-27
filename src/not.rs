use super::composite_component_logic;
use crate::nand::nand;
use crate::{BitState, ComponentGraph, DigitalComponent};
use std::collections::HashMap;

struct Not {
    dc: DigitalComponent,
}

impl Not {
    fn new() -> Not {
        let components = vec![DigitalComponent::new(2, 1, Box::new(nand))];
        let cg = ComponentGraph {
            components,
            wiring: HashMap::new(),
        };
        Not {
            dc: DigitalComponent::new(
                1,
                1,
                Box::new(composite_component_logic(
                    cg,
                    Box::new(Not::map_inputs),
                    Box::new(Not::map_outputs),
                )),
            ),
        }
    }

    fn map_inputs(inputs: &Vec<BitState>, components: &mut Vec<DigitalComponent>) {
        components[0].set_input(0, &inputs[0]);
        components[0].set_input(1, &inputs[0]);
    }

    fn map_outputs(outputs: &mut Vec<BitState>, components: &mut Vec<DigitalComponent>) {
        outputs[0] = components[0].get_output(0).clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_gate() {
        let mut not_gate = Not::new();
        not_gate.dc.set_input(0, &BitState::On);
        not_gate.dc.resolve();
        assert_eq!(not_gate.dc.get_output(0), &BitState::Off);

        not_gate.dc.set_input(0, &BitState::Off);
        assert_eq!(not_gate.dc.resolve(), true);
        assert_eq!(not_gate.dc.get_output(0), &BitState::On);
    }
}
