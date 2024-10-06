use super::composite_component_logic;
use crate::byte::{Byte, SET_BIT_IDX as BYTE_SET_VALUE_PIN_IDX};
use crate::enabler::{Enabler, ENABLE_PIN_IDX as ENABLER_ENABLE_PIN_IDX};
use crate::{BitState, ComponentGraph, ComponentId, Input, Output};
use digital_component::DigitalComponent;

pub struct Register {
    pub dc: DigitalComponent,
}

impl Register {
    pub fn new() -> Register {
        Register::named("Register")
    }

    pub fn named(name: &str) -> Register {
        let components = vec![Byte::new().dc, Enabler::new().dc];
        let mut cg = ComponentGraph::new(components);
        for pin_pair_idx in 0..IO_PINS_NUMBER {
            cg.connect(
                (ComponentId(0), Output(pin_pair_idx)),
                (ComponentId(1), Input(pin_pair_idx)),
            )
        }
        Register {
            dc: DigitalComponent::named(
                IO_PINS_NUMBER + 2,
                IO_PINS_NUMBER,
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

pub const IO_PINS_NUMBER: usize = 8;
pub const ENABLE_PIN_IDX: usize = 8;
pub const SET_VALUE_IN_IDX: usize = 9;

fn map_inputs(inputs: &Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    for input_idx in 0..IO_PINS_NUMBER {
        components[0].set_input(input_idx, &inputs[input_idx]);
    }
    components[1].set_input(ENABLER_ENABLE_PIN_IDX, &inputs[ENABLE_PIN_IDX]);
    components[0].set_input(BYTE_SET_VALUE_PIN_IDX, &inputs[SET_VALUE_IN_IDX]);
}

fn map_outputs(outputs: &mut Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    for output_idx in 0..IO_PINS_NUMBER {
        outputs[output_idx] = components[1].get_output(output_idx).clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_and_outputs_value() {
        let Register { dc: mut register } = Register::new();
        register.set_inputs(vec![
            0, 1, 0, 1, 0, 1, 0, 1, /*enable bit*/ 0, /*set value bit*/ 0,
        ]);
        register.set_input(SET_VALUE_IN_IDX, &BitState::On);
        register.resolve();
        register.set_input(SET_VALUE_IN_IDX, &BitState::Off);
        register.resolve();
        assert_eq!(register.get_outputs(), vec![0; 8]);

        register.set_input(ENABLE_PIN_IDX, &BitState::On);
        register.resolve();
        assert_eq!(register.get_outputs(), vec![0, 1, 0, 1, 0, 1, 0, 1]);
    }
}
