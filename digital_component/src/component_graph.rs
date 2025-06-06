use super::{BitState, ComponentLogic, DigitalComponent};
use crate::debug_logger::{begin_context, end_context, log};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Error, Formatter};
use std::hash::Hash;

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum NodeKind {
    ComponentInput(ComponentInput),
    ComponentOutput(ComponentOutput),
    Input(usize),
    Output(usize),
    Joint,
}

pub type ComponentInput = ComponentPin;
pub type ComponentOutput = ComponentPin;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct ComponentPin {
    component: DigitalComponentRef,
    pin: usize,
}

impl ComponentPin {
    pub fn new(component: DigitalComponentRef, pin: usize) -> ComponentPin {
        ComponentPin { component, pin }
    }
}

impl Debug for NodeKind {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_fmt(format_args!(
            "{}",
            match self {
                NodeKind::ComponentInput(ComponentPin { pin, component }) =>
                    format!("component_input({} {})", component, pin),
                NodeKind::ComponentOutput(ComponentPin { pin, component }) =>
                    format!("component_output({} {})", component, pin),
                NodeKind::Input(idx) => format!("input({})", idx),
                NodeKind::Output(idx) => format!("output({})", idx),
                NodeKind::Joint => "joint".to_string(),
            }
        ))
    }
}

pub type GraphNodeRef = usize;
pub type DigitalComponentRef = usize;

#[derive(PartialEq, Eq, Default)]
pub struct Graph {
    components: Vec<DigitalComponent>,
    nodes: Vec<NodeKind>,
    adjacency: Vec<HashSet<GraphNodeRef>>,
}

impl Graph {
    pub fn add_component(&mut self, component: DigitalComponent) -> DigitalComponentRef {
        self.components.push(component);
        self.components.len() - 1
    }

    pub fn add_edge(&mut self, a: &GraphNodeRef, b: &GraphNodeRef) {
        self.adjacency[*a].insert(*b);
        self.adjacency[*b].insert(*a);
    }

    pub fn add_node(&mut self, node_kind: NodeKind) -> GraphNodeRef {
        self.nodes.push(node_kind);
        self.adjacency.push(HashSet::default());
        self.nodes.len() - 1
    }

    fn find_disjointed_node_sets(&self) -> Vec<GraphNodeRef> {
        let mut uf_component_indices: Vec<_> = (0..self.nodes.len()).collect();
        for node_idx in 0..self.nodes.len() {
            let mut this_idx = node_idx;
            while this_idx != uf_component_indices[this_idx] {
                this_idx = uf_component_indices[this_idx];
            }
            for neighbour_idx in self.adjacency[node_idx].iter() {
                let mut that_idx = *neighbour_idx;
                while that_idx != uf_component_indices[that_idx] {
                    that_idx = uf_component_indices[that_idx];
                }
                uf_component_indices[that_idx] = this_idx;
            }
        }

        for idx in 0..uf_component_indices.len() {
            let mut root_component_idx = idx;
            let mut walk_back_indices = vec![];
            while uf_component_indices[root_component_idx] != root_component_idx {
                walk_back_indices.push(root_component_idx);
                root_component_idx = uf_component_indices[root_component_idx];
            }
            while let Some(last) = walk_back_indices.pop() {
                uf_component_indices[last] = root_component_idx;
            }
        }
        uf_component_indices
    }

    fn find_mapping(
        &self,
        uf_component_indices: &[GraphNodeRef],
        source_node_type_check: &dyn Fn(&NodeKind) -> bool,
        dest_node_type_check: &dyn Fn(&NodeKind) -> bool,
    ) -> HashMap<GraphNodeRef, HashSet<GraphNodeRef>> {
        let mut mapping = HashMap::new();

        for idx in 0..self.nodes.len() {
            if source_node_type_check(&self.nodes[idx]) {
                let node_set_idx = uf_component_indices[idx];
                let dest = uf_component_indices
                    .iter()
                    .enumerate()
                    .filter(|(_, set_idx)| **set_idx == node_set_idx)
                    .filter_map(|(neighbour_idx, _)| {
                        Some(neighbour_idx).filter(|idx| dest_node_type_check(&self.nodes[*idx]))
                    })
                    .collect::<HashSet<_>>();
                if !dest.is_empty() {
                    mapping.insert(idx, dest);
                }
            }
        }
        mapping
    }

    fn output_to_input_mapping(
        &self,
        uf_component_indices: &[GraphNodeRef],
    ) -> HashMap<GraphNodeRef, HashSet<GraphNodeRef>> {
        self.find_mapping(
            uf_component_indices,
            &|node| matches!(node, NodeKind::ComponentOutput { .. }),
            &|node| matches!(node, NodeKind::ComponentInput { .. }),
        )
    }

    fn outer_input_mapping(
        &self,
        uf_component_indices: &[usize],
    ) -> HashMap<GraphNodeRef, HashSet<GraphNodeRef>> {
        self.find_mapping(
            uf_component_indices,
            &|node| matches!(node, NodeKind::Input { .. }),
            &|node| matches!(node, NodeKind::ComponentInput { .. }),
        )
    }

    fn outer_output_mapping(
        &self,
        uf_component_indices: &[usize],
    ) -> HashMap<GraphNodeRef, HashSet<GraphNodeRef>> {
        self.find_mapping(
            uf_component_indices,
            &|node| matches!(node, NodeKind::Output { .. }),
            &|node| matches!(node, NodeKind::ComponentOutput { .. }),
        )
    }

    pub fn finalize(mut self) -> Box<ComponentLogic> {
        begin_context();
        log(format_args!("{:?}", &self));
        let uf_component_indices = self.find_disjointed_node_sets();
        let outputs_to_inputs = self.output_to_input_mapping(&uf_component_indices);
        let outer_input_mapping = self.outer_input_mapping(&uf_component_indices);
        let outer_output_mapping = self.outer_output_mapping(&uf_component_indices);

        let nodes = self.nodes;

        let mut nested_components_inputs = self
            .components
            .iter()
            .map(|component| vec![BitState::Undefined; component.get_input_num()])
            .collect::<Vec<_>>();

        let mut nested_components_outputs = self
            .components
            .iter()
            .map(|component| vec![BitState::Undefined; component.get_output_num()])
            .collect::<Vec<_>>();

        let component_logic = move |input_bits: &[BitState], output_bits: &mut [BitState]| {
            begin_context();
            let node_to_idx = nodes
                .iter()
                .enumerate()
                .map(|(idx, node)| (node, idx))
                .collect::<HashMap<_, _>>();

            let mut dirty_components = propagate_outer_input(
                &nodes,
                &mut nested_components_inputs,
                input_bits,
                &outer_input_mapping,
            );

            while let Some(nested_component) = dirty_components.pop() {
                let outputs = &mut nested_components_outputs[nested_component];
                let inputs = &nested_components_inputs[nested_component];
                self.components[nested_component].get_func()(inputs, outputs);
                log(format_args!(
                    "for component {} : {:?} -> {:?}",
                    nested_component, &inputs, &outputs
                ));

                // propagate the signal to dependant components
                for (pin, output_bit) in outputs.iter().enumerate() {
                    let output_pin = ComponentOutput::new(nested_component, pin);
                    let output_pin_idx = node_to_idx
                        .get(&NodeKind::ComponentOutput(output_pin.clone()))
                        .unwrap_or_else(|| {
                            panic!(
                                "we processed this output, it must be present among the nodes {:?}",
                                &output_pin
                            )
                        });
                    let connected_input_indices = outputs_to_inputs.get(output_pin_idx);
                    for connected_input_node in connected_input_indices
                        .iter()
                        .flat_map(|x| *x)
                        .map(|input_idx| &nodes[*input_idx])
                    {
                        if let NodeKind::ComponentInput(connected_input_node) = connected_input_node
                        {
                            let connected_inputs_bits = nested_components_inputs
                                .get_mut(connected_input_node.component)
                                .unwrap();
                            if connected_inputs_bits[connected_input_node.pin] != *output_bit {
                                connected_inputs_bits[connected_input_node.pin] = *output_bit;
                                dirty_components.push(connected_input_node.component);
                            }
                        } else {
                            panic!("this is impossible");
                        }
                    }
                }
            }

            propagate_to_outer_output(
                &nodes,
                &mut nested_components_outputs,
                output_bits,
                &outer_output_mapping,
            );

            log(format_args!("output mapping: {:?}", &outer_output_mapping));
            log(format_args!("output bits: {:?}", &output_bits));
            end_context();
        };
        end_context();
        Box::new(component_logic)
    }
}

fn propagate_outer_input(
    nodes: &[NodeKind],
    nested_components_state: &mut [Vec<BitState>],
    input_bits: &[BitState],
    outer_input_mapping: &HashMap<GraphNodeRef, HashSet<GraphNodeRef>>,
) -> Vec<DigitalComponentRef> {
    let mut touched_input_components = vec![];
    let mut touched_input_set = HashSet::new();
    for (outer_input_ref, nested_input_ref) in
        outer_input_mapping
            .iter()
            .flat_map(|(input_idx, node_set)| {
                node_set.iter().map(move |node_ref| (input_idx, node_ref))
            })
    {
        if let (
            NodeKind::Input(input_idx),
            NodeKind::ComponentInput(ComponentInput { component, pin }),
        ) = (
            nodes[*outer_input_ref].clone(),
            nodes[*nested_input_ref].clone(),
        ) {
            let inputs = &mut nested_components_state[component];
            inputs[pin] = input_bits[input_idx];
            if !touched_input_set.contains(&component) {
                touched_input_components.push(component);
                touched_input_set.insert(component);
            }
        }
    }
    touched_input_components
}

fn propagate_to_outer_output(
    nodes: &[NodeKind],
    nested_components_outputs: &mut [Vec<BitState>],
    output_bits: &mut [BitState],
    outer_output_mapping: &HashMap<GraphNodeRef, HashSet<GraphNodeRef>>,
) {
    let mut new_output = vec![BitState::Undefined; output_bits.len()];
    for (outer_output_ref, nested_output_ref) in
        outer_output_mapping
            .iter()
            .flat_map(|(output_idx, node_set)| {
                node_set.iter().map(move |node_ref| (output_idx, node_ref))
            })
    {
        if let (
            NodeKind::Output(output_idx),
            NodeKind::ComponentOutput(ComponentOutput { component, pin }),
        ) = (
            nodes[*outer_output_ref].clone(),
            nodes[*nested_output_ref].clone(),
        ) {
            log(format_args!(
                "copying from component {component} pin {pin} to output {output_idx}"
            ));
            let outputs = &mut nested_components_outputs[component];
            let new_output_value = match (outputs[pin], new_output[output_idx]) {
                (BitState::On, _) => BitState::On,
                (BitState::Off, BitState::Undefined) => BitState::Off,
                _ => output_bits[output_idx],
            };
            new_output[output_idx] = new_output_value;
        }
    }
    for (output_bit, new_output_bit) in output_bits.iter_mut().zip(new_output.iter()) {
        *output_bit = *new_output_bit;
    }
}

impl Debug for Graph {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for (node_idx, node) in self.nodes.iter().enumerate() {
            f.write_fmt(format_args!("{:?}_{:?} -> [ ", node_idx, node))?;

            let mut neighbours = self.adjacency[node_idx]
                .iter()
                .map(|neighbour_idx| {
                    format!("{:?}_{:?}", neighbour_idx, self.nodes[*neighbour_idx],)
                })
                .collect::<Vec<_>>();

            neighbours.sort();
            f.write_str(&neighbours.join(", "))?;
            f.write_str("]\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn test(input: &[BitState], output: &mut [BitState]) {
        for (input, output) in input.iter().zip(output.iter_mut()) {
            *output = match input {
                BitState::On => BitState::Off,
                BitState::Off => BitState::On,
                BitState::Undefined => BitState::Undefined,
            }
        }
    }

    #[test]
    fn builds_graph() {
        let comp = DigitalComponent::new(1, 1, Box::new(test));

        let mut graph = Graph::default();
        let comp = graph.add_component(comp);
        let a = NodeKind::ComponentInput(ComponentInput::new(comp, 0));
        let b = NodeKind::Joint;
        let c = NodeKind::Joint;
        let d = NodeKind::Joint;
        let a_node = graph.add_node(a);
        let b_node = graph.add_node(b);
        let _c_node = graph.add_node(c);
        let d_node = graph.add_node(d);
        graph.add_edge(&a_node, &b_node);
        graph.add_edge(&d_node, &b_node);

        assert_eq!(
            format!("{graph:?}"),
            "\
            0_component_input(0 0) -> [ 1_joint]\n\
            1_joint -> [ 0_component_input(0 0), 3_joint]\n\
            2_joint -> [ ]\n\
            3_joint -> [ 1_joint]\n\
            "
        );
    }

    #[test]
    fn finds_disjointed_node_sets() {
        let comp = DigitalComponent::new(1, 1, Box::new(test));
        let mut graph = Graph::default();
        let a = NodeKind::ComponentInput(ComponentInput::new(graph.add_component(comp), 0));
        let a_node = graph.add_node(a);
        let b_node = graph.add_node(NodeKind::Joint);
        let _c_node = graph.add_node(NodeKind::Joint);
        let d_node = graph.add_node(NodeKind::Joint);
        let _e_node = graph.add_node(NodeKind::Joint);
        graph.add_edge(&a_node, &b_node);
        graph.add_edge(&d_node, &b_node);

        assert_eq!(graph.find_disjointed_node_sets(), vec![0, 0, 2, 0, 4]);
    }

    #[test]
    fn generates_output_to_input_mapping() {
        let reg_1 = DigitalComponent::new(2, 2, Box::new(test));
        let reg_2 = DigitalComponent::new(2, 2, Box::new(test));

        let mut graph = Graph::default();
        let reg_1 = graph.add_component(reg_1);
        let _reg_1_input_0 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(reg_1, 0)));
        let _reg_1_input_1 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(reg_1, 1)));
        let reg_1_output_0 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(reg_1, 0)));
        let reg_1_output_1 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(reg_1, 1)));

        let joint = graph.add_node(NodeKind::Joint);

        let reg_2 = graph.add_component(reg_2);
        let reg_2_input_0 = graph.add_node(NodeKind::ComponentInput(ComponentInput::new(reg_2, 0)));
        let reg_2_input_1 = graph.add_node(NodeKind::ComponentInput(ComponentInput::new(reg_2, 1)));
        let _reg_2_output_0 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(reg_2, 0)));
        let _reg_2_output_1 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(reg_2, 1)));

        graph.add_edge(&reg_1_output_0, &joint);
        graph.add_edge(&joint, &reg_2_input_0);
        graph.add_edge(&reg_1_output_1, &reg_2_input_1);

        let mut expected = HashMap::new();
        let mut outputs_connected_to_reg1_output_0 = HashSet::new();
        outputs_connected_to_reg1_output_0.insert(reg_2_input_0);
        expected.insert(reg_1_output_0, outputs_connected_to_reg1_output_0);

        let mut outputs_connected_to_reg1_output_1 = HashSet::new();
        outputs_connected_to_reg1_output_1.insert(reg_2_input_1);
        expected.insert(reg_1_output_1, outputs_connected_to_reg1_output_1);

        let mapping = graph.output_to_input_mapping(&graph.find_disjointed_node_sets());

        assert_eq!(mapping, expected);
    }

    #[test]
    fn generates_outer_input_mapping() {
        let reg_1 = DigitalComponent::new(2, 2, Box::new(test));
        let reg_2 = DigitalComponent::new(2, 2, Box::new(test));

        let mut graph = Graph::default();
        let reg_1 = graph.add_component(reg_1);

        let reg_1_input_0 = graph.add_node(NodeKind::ComponentInput(ComponentInput::new(reg_1, 0)));
        let _reg_1_input_1 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(reg_1, 1)));

        let reg_2 = graph.add_component(reg_2);
        let _reg_2_input_0 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(reg_2, 0)));
        let reg_2_input_1 = graph.add_node(NodeKind::ComponentInput(ComponentInput::new(reg_2, 1)));
        let outer_input_0 = graph.add_node(NodeKind::Input(0));
        let outer_input_1 = graph.add_node(NodeKind::Input(1));
        graph.add_edge(&outer_input_0, &reg_1_input_0);
        graph.add_edge(&outer_input_1, &reg_2_input_1);

        let expected = [
            (
                outer_input_0,
                [reg_1_input_0].into_iter().collect::<HashSet<_>>(),
            ),
            (
                outer_input_1,
                [reg_2_input_1].into_iter().collect::<HashSet<_>>(),
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let mapping = graph.outer_input_mapping(&graph.find_disjointed_node_sets());

        assert_eq!(mapping, expected);
    }

    #[test]
    fn generates_outer_output_mapping() {
        let reg_1 = DigitalComponent::new(2, 2, Box::new(test));
        let reg_2 = DigitalComponent::new(2, 2, Box::new(test));

        let mut graph = Graph::default();
        let reg_1 = graph.add_component(reg_1);
        let reg_1_output_0 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(reg_1, 0)));
        let _reg_1_output_1 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(reg_1, 1)));

        let reg_2 = graph.add_component(reg_2);
        let _reg_2_output_0 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(reg_2, 0)));
        let reg_2_output_1 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(reg_2, 1)));
        let outer_output_0 = graph.add_node(NodeKind::Output(0));
        let outer_output_1 = graph.add_node(NodeKind::Output(1));
        graph.add_edge(&outer_output_0, &reg_1_output_0);
        graph.add_edge(&outer_output_1, &reg_2_output_1);

        let expected = [
            (
                outer_output_0,
                vec![reg_1_output_0].into_iter().collect::<HashSet<_>>(),
            ),
            (
                outer_output_1,
                vec![reg_2_output_1].into_iter().collect::<HashSet<_>>(),
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let mapping = graph.outer_output_mapping(&graph.find_disjointed_node_sets());

        assert_eq!(mapping, expected);
    }

    #[test]
    fn converts_graph_into_component_logic() {
        let comp_0 = DigitalComponent::new(2, 2, Box::new(test));
        let mut graph = Graph::default();
        let comp_0 = graph.add_component(comp_0);
        let comp_0_input_0 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(comp_0, 0)));
        let comp_0_input_1 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(comp_0, 1)));

        let comp_0_output_0 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(comp_0, 0)));
        let comp_0_output_1 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(comp_0, 1)));

        let comp_1 = graph.add_component(DigitalComponent::new(1, 1, Box::new(test)));
        let comp_1_input_0 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(comp_1, 0)));

        let comp_1_output_0 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(comp_1, 0)));

        let input_0 = graph.add_node(NodeKind::Input(0));
        let input_1 = graph.add_node(NodeKind::Input(1));

        let output_0 = graph.add_node(NodeKind::Output(0));
        let output_1 = graph.add_node(NodeKind::Output(1));

        graph.add_edge(&input_0, &comp_0_input_0);
        graph.add_edge(&input_1, &comp_0_input_1);

        graph.add_edge(&comp_0_output_1, &comp_1_input_0);

        graph.add_edge(&output_0, &comp_0_output_0);
        graph.add_edge(&output_1, &comp_1_output_0);

        let mut comp_logic = graph.finalize();
        let mut output = vec![BitState::Undefined; 2];
        comp_logic(&[BitState::On, BitState::Off], &mut output);
        assert_eq!(output, vec![BitState::Off, BitState::Off]);
    }
}
