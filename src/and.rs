use std::collections::HashMap;

use super::composite_component_logic;
use crate::digital_component::DigitalComponent;
use crate::nand::Nand;
use crate::not::not;
use crate::{BitState, ComponentGraph, ComponentId, Input, Output};

pub fn make_and(name: &str) -> DigitalComponent {
    let components = vec![Nand::named("nand #1").dc, not("not #1")];
    let mut cg = ComponentGraph {
        components,
        wiring: HashMap::new(),
    };
    cg.connect((ComponentId(0), Output(0)), (ComponentId(1), Input(0)));
    DigitalComponent::named(
        2,
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
    components[0].set_input(1, &inputs[1]);
}

fn map_outputs(outputs: &mut Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    outputs[0] = components[1].get_output(0).clone();
}

pub fn cascade_and(name: &str, inputs_num: usize) -> DigitalComponent {
    assert!(
        inputs_num > 1,
        "A cascade of and gates can have no fewer than 2 inputs. Got {inputs_num}"
    );
    let components = (0..(inputs_num - 1))
        .map(|num| make_and(&format!("and #{num}")))
        .collect();
    let mut cg = ComponentGraph::new(components);
    for component_idx in 0..(inputs_num - 2) {
        cg.connect(
            (ComponentId(component_idx), Output(0)),
            (ComponentId(component_idx + 1), Input(1)),
        );
    }
    DigitalComponent::named(
        inputs_num,
        1,
        Box::new(composite_component_logic(
            cg,
            Box::new(map_cascaded_inputs),
            Box::new(map_cascaded_outputs),
        )),
        name,
    )
}

fn map_cascaded_inputs(inputs: &Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    for (input, component) in inputs.iter().zip(components.iter_mut()) {
        component.set_input(0, input);
    }
    components[0].set_input(1, inputs.last().unwrap_or(&BitState::Undefined));
}

fn map_cascaded_outputs(outputs: &mut Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    outputs[0] = components
        .last()
        .map(|last_and_gate| last_and_gate.get_output(0).clone())
        .unwrap_or(BitState::Undefined);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and() {
        let mut and = make_and("");
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

    #[test]
    fn cascaded_ands() {
        let mut and = cascade_and("", 3);
        and.set_inputs(vec![0, 0, 0]);
        and.resolve();
        assert_eq!(*and.get_output(0), BitState::Off);

        and.set_inputs(vec![1, 0, 0]);
        and.resolve();
        assert_eq!(*and.get_output(0), BitState::Off);

        and.set_inputs(vec![1, 1, 0]);
        and.resolve();
        assert_eq!(*and.get_output(0), BitState::Off);

        println!("and for setting all to on");
        and.set_inputs(vec![1, 1, 1]);
        and.resolve();
        assert_eq!(*and.get_output(0), BitState::On);
    }
}
