use std::collections::HashMap;

use super::composite_component_logic;
use crate::digital_component::DigitalComponent;
use crate::nand::Nand;
use crate::not::Not;
use crate::{BitState, ComponentGraph, ComponentId, Input, Output};

pub struct And {
    pub dc: DigitalComponent,
}

impl And {
    pub fn new() -> And {
        And::named("")
    }

    pub fn named(name: &str) -> And {
        let components = vec![Nand::named("nand #1").dc, Not::new().dc];
        let mut cg = ComponentGraph {
            components,
            wiring: HashMap::new(),
        };
        cg.connect((ComponentId(0), Output(0)), (ComponentId(1), Input(0)));
        And {
            dc: DigitalComponent::named(
                2,
                1,
                Box::new(composite_component_logic(
                    cg,
                    Box::new(map_inputs),
                    Box::new(map_outputs),
                )),
                name,
            ),
        }
    }
}

fn map_inputs(inputs: &Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    components[0].set_input(0, &inputs[0]);
    components[0].set_input(1, &inputs[1]);
}

fn map_outputs(outputs: &mut Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    outputs[0] = components[1].get_output(0).clone();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and() {
        let And { dc: mut and } = And::new();
        and.set_input(0, &BitState::On);
        and.set_input(1, &BitState::On);
        and.resolve();
        assert_eq!(*and.get_output(0), BitState::On);

        and.set_input(0, &BitState::On);
        and.set_input(1, &BitState::Off);
        and.resolve();
        assert_eq!(*and.get_output(0), BitState::Off);

        and.set_input(0, &BitState::On);
        and.set_input(1, &BitState::Off);
        and.resolve();
        assert_eq!(*and.get_output(0), BitState::Off);

        and.set_input(0, &BitState::Off);
        and.set_input(1, &BitState::Off);
        and.resolve();
        assert_eq!(*and.get_output(0), BitState::Off);
    }
}
