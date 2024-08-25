use std::cmp::Eq;
use std::collections::{HashMap, HashSet, VecDeque};

fn main() {}

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

    fn map_inputs(inputs: &Vec<bool>, components: &mut Vec<DigitalComponent>) {
        components[0].set_input(0, inputs[0]);
        components[0].set_input(1, inputs[0]);
    }

    fn map_outputs(outputs: &mut Vec<bool>, components: &mut Vec<DigitalComponent>) {
        outputs[0] = components[0].get_output(0);
    }
}

struct DigitalComponent {
    inputs: Vec<bool>,
    outputs: Vec<bool>,
    func: Box<ComponentLogic>,
}

struct Output(usize);
struct Input(usize);

#[derive(Eq, PartialEq, Hash)]
struct ComponentId(usize);

struct ComponentGraph {
    components: Vec<DigitalComponent>,
    wiring: HashMap<(ComponentId, Output), HashSet<(ComponentId, Input)>>,
}

fn composite_component_logic(
    mut component_graph: ComponentGraph,
    mut inputs_mapping: Box<dyn FnMut(&Vec<bool>, &mut Vec<DigitalComponent>) -> ()>,
    mut outputs_mapping: Box<dyn FnMut(&mut Vec<bool>, &mut Vec<DigitalComponent>) -> ()>,
) -> Box<ComponentLogic> {
    Box::new(move |input: &Vec<bool>, output: &mut Vec<bool>| {
        let mut events: VecDeque<ComponentId> = (0..component_graph.components.len())
            .map(ComponentId)
            .collect();

        let component_graph = &mut component_graph;
        (*inputs_mapping)(input, &mut component_graph.components);
        //set inputs to some of the inner components, and change the outputs accordingly
        let mut changed = false;
        while let Some(ComponentId(component_idx)) = events.pop_front() {
            let component = &mut component_graph.components[component_idx];
            let state_changed = component.resolve();
            changed = state_changed || changed;
            if state_changed {
                //todo walk down the graph and update adjacent components
            }
        }
        (*outputs_mapping)(output, &mut component_graph.components);
        changed
    })
}

/// Maps vector of input to vector of outputs
///
/// Return [`true`] if output values changed
type ComponentLogic = dyn FnMut(&Vec<bool>, &mut Vec<bool>) -> bool;

impl DigitalComponent {
    fn new(input_number: usize, output_number: usize, func: Box<ComponentLogic>) -> Self {
        let mut dc = DigitalComponent {
            inputs: vec![false; input_number],
            outputs: vec![false; output_number],
            func,
        };
        dc.resolve();
        dc
    }

    fn set_input(&mut self, idx: usize, value: bool) {
        self.inputs[idx] = value;
    }

    fn get_output(&self, idx: usize) -> bool {
        self.outputs[idx]
    }

    fn resolve(&mut self) -> bool {
        (self.func)(&self.inputs, &mut self.outputs)
    }
}

fn nand(input: &Vec<bool>, output: &mut Vec<bool>) -> bool {
    assert!(input.len() == 2, "NAND gate must have exactly two inputs");
    assert!(output.len() == 1, "NAND gate must have exactly one ouput");
    let previous_value = output[0];
    output[0] = match (input[0], input[1]) {
        (true, true) => false,
        _ => true,
    };
    previous_value != output[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nand_gate() {
        let mut nand_component = DigitalComponent::new(2, 1, Box::new(nand));
        assert_eq!(nand_component.get_output(0), true);
        nand_component.set_input(0, true);
        assert_eq!(nand_component.resolve(), false);
        assert_eq!(nand_component.get_output(0), true);

        nand_component.set_input(1, true);
        assert_eq!(nand_component.resolve(), true);
        assert_eq!(nand_component.get_output(0), false);
    }

    #[test]
    fn not_gate() {
        let mut not_gate = Not::new();
        not_gate.dc.set_input(0, false);
        not_gate.dc.resolve();
        assert_eq!(not_gate.dc.get_output(0), true);

        not_gate.dc.set_input(0, true);
        assert_eq!(not_gate.dc.resolve(), true);
        assert_eq!(not_gate.dc.get_output(0), false);
    }
}
