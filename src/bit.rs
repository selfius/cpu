use std::collections::HashMap;

use crate::{
    composite_component_logic, BitState, ComponentGraph, ComponentId, DigitalComponent, Input,
    Output,
};

use crate::nand::Nand;

pub struct Bit {
    pub dc: DigitalComponent,
}

impl Bit {
    pub fn new() -> Bit {
        let components = (0..=3)
            .map(|num| Nand::named(&num.to_string()).dc())
            .collect();
        let wiring = HashMap::new();
        let mut cg = ComponentGraph { components, wiring };
        cg.connect((ComponentId(0), Output(0)), (ComponentId(1), Input(0)));
        cg.connect((ComponentId(0), Output(0)), (ComponentId(2), Input(0)));
        cg.connect((ComponentId(1), Output(0)), (ComponentId(3), Input(1)));
        cg.connect((ComponentId(2), Output(0)), (ComponentId(3), Input(0)));
        cg.connect((ComponentId(3), Output(0)), (ComponentId(2), Input(1)));

        Bit {
            dc: DigitalComponent::new(
                2,
                1,
                Box::new(composite_component_logic(
                    cg,
                    Box::new(Bit::map_inputs),
                    Box::new(Bit::map_outputs),
                )),
            ),
        }
    }

    fn map_inputs(inputs: &Vec<BitState>, components: &mut Vec<DigitalComponent>) {
        components[0].set_input(0, &inputs[0]);
        components[0].set_input(1, &inputs[1]);
        components[1].set_input(1, &inputs[1]);
    }

    fn map_outputs(outputs: &mut Vec<BitState>, components: &mut Vec<DigitalComponent>) {
        outputs[0] = components[2].get_output(0).clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_bit() {
        let mut bit = Bit::new();

        bit.dc.set_input(0, &BitState::Off); // value = 0
        bit.dc.set_input(1, &BitState::On); // set bit
        bit.dc.resolve();
        assert_eq!(bit.dc.get_output(0), &BitState::Off);

        bit.dc.set_input(1, &BitState::Off); // set bit no longer needed
        bit.dc.resolve();
        assert_eq!(bit.dc.get_output(0), &BitState::Off);

        bit.dc.set_input(0, &BitState::On); // value = 1
        bit.dc.set_input(1, &BitState::On); //
        bit.dc.resolve();

        bit.dc.set_input(1, &BitState::Off); // set bit no longer needed
        bit.dc.resolve();
        assert_eq!(bit.dc.get_output(0), &BitState::On);
    }
}
