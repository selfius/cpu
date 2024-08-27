use std::cmp::Eq;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

mod bit;
mod byte;
mod nand;
mod not;

fn main() {}

#[derive(Clone, Debug, PartialEq)]
enum BitState {
    On,
    Off,
    Undefined,
}

struct DigitalComponent {
    name: String,
    inputs: Vec<BitState>,
    outputs: Vec<BitState>,
    func: Box<ComponentLogic>,
}

impl fmt::Display for DigitalComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inputs: String = self
            .inputs
            .iter()
            .map(|signal| match signal {
                BitState::On => "1, ",
                BitState::Off => "0, ",
                BitState::Undefined => "u, ",
            })
            .collect();

        let outputs: String = self
            .outputs
            .iter()
            .map(|signal| match signal {
                BitState::On => "1, ",
                BitState::Off => "0, ",
                BitState::Undefined => "u, ",
            })
            .collect();
        let name = match self.name.is_empty() {
            true => "unnamed",
            false => &self.name,
        };

        write!(f, "{} inputs[{}] outputs[{}]", name, inputs, outputs)
    }
}

#[derive(Eq, PartialEq, Hash)]
struct Output(usize);

#[derive(Eq, PartialEq, Hash)]
struct Input(usize);

#[derive(Eq, PartialEq, Hash)]
struct ComponentId(usize);

struct ComponentGraph {
    components: Vec<DigitalComponent>,
    wiring: HashMap<(ComponentId, Output), HashSet<(ComponentId, Input)>>,
}

impl ComponentGraph {
    fn connect(&mut self, output: (ComponentId, Output), input: (ComponentId, Input)) {
        let input_set = self.wiring.entry(output).or_insert(HashSet::new());
        input_set.insert(input);
    }
}

fn composite_component_logic(
    mut component_graph: ComponentGraph,
    mut inputs_mapping: Box<dyn FnMut(&Vec<BitState>, &mut Vec<DigitalComponent>) -> ()>,
    mut outputs_mapping: Box<dyn FnMut(&mut Vec<BitState>, &mut Vec<DigitalComponent>) -> ()>,
) -> Box<ComponentLogic> {
    Box::new(move |input: &Vec<BitState>, output: &mut Vec<BitState>| {
        let mut events: VecDeque<ComponentId> = (0..component_graph.components.len())
            .map(ComponentId)
            .collect();

        let component_graph = &mut component_graph;
        (*inputs_mapping)(input, &mut component_graph.components);
        // set inputs to some of the inner components, and change the outputs accordingly
        let mut changed = false;
        while let Some(ComponentId(component_idx)) = events.pop_front() {
            let component = &mut component_graph.components[component_idx];

            let state_changed = component.resolve();
            println!("{} [changed: {}]", component, state_changed);

            changed = state_changed || changed;
            if state_changed {
                println!("change detected for {component_idx}");
                for output_pin in 0..component.outputs.len() {
                    let value_to_propagate = component_graph.components[component_idx]
                        .get_output(output_pin)
                        .clone();
                    for (ComponentId(connected_input_component_id), Input(input_pin)) in
                        component_graph
                            .wiring
                            .get(&(ComponentId(component_idx), Output(output_pin)))
                            .unwrap_or(&HashSet::new())
                            .iter()
                    {
                        println!(
                            "setting {connected_input_component_id}:{input_pin} to {:?}",
                            &value_to_propagate
                        );
                        let connected_component =
                            &mut component_graph.components[*connected_input_component_id];
                        connected_component.set_input(*input_pin, &value_to_propagate);
                        events.push_back(ComponentId(*connected_input_component_id));
                    }
                }
            }
        }
        //maps the state of the inner components to the outer outputs
        (*outputs_mapping)(output, &mut component_graph.components);
        changed
    })
}

/// Maps vector of input to vector of outputs
///
/// Return [`true`] if output values changed
type ComponentLogic = dyn FnMut(&Vec<BitState>, &mut Vec<BitState>) -> bool;

impl DigitalComponent {
    fn new(input_number: usize, output_number: usize, func: Box<ComponentLogic>) -> Self {
        DigitalComponent::named(input_number, output_number, func, "")
    }

    fn named(
        input_number: usize,
        output_number: usize,
        func: Box<ComponentLogic>,
        name: &str,
    ) -> Self {
        let mut dc = DigitalComponent {
            name: String::from(name),
            inputs: vec![BitState::Undefined; input_number],
            outputs: vec![BitState::Undefined; output_number],
            func,
        };
        dc.resolve();
        dc
    }

    fn set_input(&mut self, idx: usize, value: &BitState) {
        self.inputs[idx] = value.clone();
    }

    fn set_inputs(&mut self, values: Vec<u8>) {
        assert!(
            values.len() == self.inputs.len(),
            "Expected exactly {} inputs",
            self.inputs.len()
        );
        for (idx, input) in self.inputs.iter_mut().enumerate() {
            *input = match values[idx] {
                0 => BitState::Off,
                1 => BitState::On,
                x => panic!(
                    "Values can be comprised only of 1s and 0s. Got {} instead",
                    x
                ),
            }
        }
    }

    fn get_output(&self, idx: usize) -> &BitState {
        &self.outputs[idx]
    }

    fn get_outputs(&self) -> Vec<u8> {
        self.outputs
            .iter()
            .map(|bit| match bit {
                BitState::On => 1,
                BitState::Off => 0,
                _ => panic!("Some outputs are in undefined state. Can't serialize to 1s and 0s"),
            })
            .collect()
    }

    fn resolve(&mut self) -> bool {
        (self.func)(&self.inputs, &mut self.outputs)
    }
}
