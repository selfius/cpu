use std::collections::HashMap;

use crate::{composite_component_logic, BitState, ComponentGraph, DigitalComponent};

use crate::bit::Bit;

pub struct Byte {
    pub dc: DigitalComponent,
}

impl Byte {
    pub fn new() -> Byte {
        let components = (0..IO_PINS_NUMBER).map(|_| Bit::new().dc).collect();
        Byte {
            dc: DigitalComponent::new(
                IO_PINS_NUMBER + 1, // general io pins + set value bit
                IO_PINS_NUMBER,
                Box::new(composite_component_logic(
                    ComponentGraph {
                        components,
                        wiring: HashMap::new(),
                    },
                    Box::new(map_inputs),
                    Box::new(map_outputs),
                )),
            ),
        }
    }
}

const IO_PINS_NUMBER: usize = 8;
pub const SET_BIT_IDX: usize = 8;

fn map_inputs(inputs: &Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    for input_idx in 0..IO_PINS_NUMBER {
        components[input_idx].set_input(0, &inputs[input_idx]);
        components[input_idx].set_input(1, &inputs[SET_BIT_IDX])
    }
}

fn map_outputs(outputs: &mut Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    for output_idx in 0..IO_PINS_NUMBER {
        outputs[output_idx] = components[output_idx].get_output(0).clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_byte() {
        let Byte { dc: mut byte } = Byte::new();
        byte.set_inputs(vec![0, 0, 0, 0, 0, 0, 0, 0, 1]);
        byte.resolve();
        byte.set_input(SET_BIT_IDX, &BitState::Off);
        byte.resolve();
        assert_eq!(byte.get_outputs(), vec![0, 0, 0, 0, 0, 0, 0, 0]);

        byte.set_inputs(vec![0, 1, 0, 1, 0, 1, 0, 1, 1]);
        byte.resolve();
        byte.set_input(SET_BIT_IDX, &BitState::Off);
        byte.resolve();
        assert_eq!(byte.get_outputs(), vec![0, 1, 0, 1, 0, 1, 0, 1]);
    }
}
