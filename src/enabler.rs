use std::collections::HashMap;

use super::composite_component_logic;
use crate::and::make_and;
use crate::digital_component::DigitalComponent;
use crate::{BitState, ComponentGraph};

pub struct Enabler {
    pub dc: DigitalComponent,
}

impl Enabler {
    pub fn new() -> Enabler {
        let components = (0..IO_PINS_NUMBER)
            .map(|num| make_and(&format!("and # {}", num)))
            .collect();
        let cg = ComponentGraph {
            components,
            wiring: HashMap::new(),
        };
        Enabler {
            dc: DigitalComponent::new(
                IO_PINS_NUMBER + 1,
                IO_PINS_NUMBER,
                Box::new(composite_component_logic(
                    cg,
                    Box::new(map_inputs),
                    Box::new(map_outputs),
                )),
            ),
        }
    }
}

const IO_PINS_NUMBER: usize = 8;
pub const ENABLE_PIN_IDX: usize = 8;

fn map_inputs(inputs: &Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    for input_idx in 0..IO_PINS_NUMBER {
        components[input_idx].set_input(0, &inputs[input_idx]);
        components[input_idx].set_input(1, &inputs[ENABLE_PIN_IDX]);
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
    fn can_enable_bytes() {
        let Enabler { dc: mut enabler } = Enabler::new();
        enabler.set_inputs(vec![1; 9]);
        enabler.set_input(ENABLE_PIN_IDX, &BitState::Off);
        enabler.resolve();
        assert_eq!(enabler.get_outputs(), vec![0; 8]);

        enabler.set_input(ENABLE_PIN_IDX, &BitState::On);
        enabler.resolve();
        assert_eq!(enabler.get_outputs(), vec![1; 8]);
    }
}
