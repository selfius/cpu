use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::Range;

mod and;
mod bit;
mod byte;
mod decoder;
mod enabler;
mod intersections;
mod nand;
mod not;
mod register;

use digital_component::{BitState, ComponentId, ComponentLogic, DigitalComponent, Input, Output};

fn main() {
    //just to shut up clippy for the time being
    decoder::decoder("test", 3);
}

struct ComponentGraph {
    components: Vec<DigitalComponent>,
    wiring: HashMap<(ComponentId, Output), HashSet<(ComponentId, Input)>>,
}

impl ComponentGraph {
    pub fn new(components: Vec<DigitalComponent>) -> ComponentGraph {
        ComponentGraph {
            components,
            wiring: HashMap::new(),
        }
    }

    fn connect(&mut self, output: (ComponentId, Output), input: (ComponentId, Input)) {
        let (ComponentId(output_component_id), Output(output_idx)) = output;
        assert!(
            self.components[output_component_id].get_outputs_num() > output_idx,
            "{} component has only {} outputs. Got {}.",
            self.components[output_component_id].name(),
            self.components[output_component_id].get_outputs_num(),
            output_idx
        );
        let (ComponentId(input_component_id), Input(input_idx)) = input;
        assert!(
            self.components[input_component_id].get_input_num() > input_idx,
            "{} component has only {} inputs. Got {}.",
            self.components[input_component_id].name(),
            self.components[input_component_id].get_input_num(),
            input_idx
        );

        assert!(self.components[input.0 .0].get_input_num() > input.1 .0);

        let input_set = self.wiring.entry(output).or_default();
        input_set.insert(input);
    }

    fn connect_range(
        &mut self,
        outputs: (ComponentId, Range<usize>),
        inputs: (ComponentId, Range<usize>),
    ) {
        for (output, input) in outputs.1.zip(inputs.1) {
            let input_set = self
                .wiring
                .entry((outputs.0.clone(), Output(output)))
                .or_default();
            input_set.insert((inputs.0.clone(), Input(input)));
        }
    }
}

type InputMappingFunc = dyn FnMut(&Vec<BitState>, &mut Vec<DigitalComponent>);
type OutputMappingFunc = dyn FnMut(&mut Vec<BitState>, &mut Vec<DigitalComponent>);

fn composite_component_logic(
    mut component_graph: ComponentGraph,
    mut inputs_mapping: Box<InputMappingFunc>,
    mut outputs_mapping: Box<OutputMappingFunc>,
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
                for output_pin in 0..component.get_outputs_num() {
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
