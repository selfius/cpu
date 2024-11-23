use super::{ComponentLogic, DigitalComponent};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::rc::Rc;

#[derive(Eq, PartialEq)]
pub enum NodeKind<'a> {
    ComponentInput(ComponentInput<'a>),
    ComponentOutput(ComponentOutput<'a>),
    Input(usize),
    Output(usize),
    Joint,
}

pub type ComponentInput<'a> = ComponentPin<'a>;
pub type ComponentOutput<'a> = ComponentPin<'a>;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ComponentPin<'a> {
    component: Rc<DigitalComponent<'a>>,
    pin: usize,
}

impl<'a> ComponentPin<'a> {
    pub fn new(component: &Rc<DigitalComponent<'a>>, pin: usize) -> ComponentPin<'a> {
        ComponentPin {
            component: Rc::clone(component),
            pin,
        }
    }
}

impl Debug for NodeKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!(
            "{}",
            match self {
                NodeKind::ComponentInput(ComponentPin { pin, .. }) =>
                    format!("component_input({})", pin),
                NodeKind::ComponentOutput(ComponentPin { pin, .. }) =>
                    format!("component_output({})", pin),
                NodeKind::Input(idx) => format!("input({})", idx),
                NodeKind::Output(idx) => format!("output({})", idx),
                NodeKind::Joint => "joint".to_string(),
            }
        ))
    }
}

pub struct GraphNode<'a> {
    idx: usize,
    kind: NodeKind<'a>,
    neighbours: HashSet<GraphNodeRef<'a>>,
}

impl GraphNode<'_> {
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }
}

#[derive(Clone)]
pub struct GraphNodeRef<'a> {
    node: Rc<RefCell<GraphNode<'a>>>,
}

impl Debug for GraphNodeRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!(
            "{:?}_{:?}",
            self.node.borrow().kind,
            ptr::addr_of!(self.node)
        ))
    }
}

impl PartialEq for GraphNodeRef<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        ptr::addr_eq(&*self.node, &*rhs.node)
    }
}

impl Eq for GraphNodeRef<'_> {}

impl Hash for GraphNodeRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::addr_of!(self).hash(state);
    }
}

#[derive(PartialEq, Eq, Default)]
pub struct Graph<'a> {
    components: HashSet<Rc<DigitalComponent<'a>>>,
    adjacency: Vec<GraphNodeRef<'a>>,
}

impl<'a> Graph<'a> {
    pub fn add_edge(&mut self, a: &mut GraphNodeRef<'a>, b: &mut GraphNodeRef<'a>) {
        a.node.borrow_mut().neighbours.insert(b.clone());
        b.node.borrow_mut().neighbours.insert(a.clone());
    }

    pub fn add_node(&mut self, node_kind: NodeKind<'a>) -> GraphNodeRef<'a> {
        match &node_kind {
            NodeKind::ComponentInput(ComponentPin { component, .. })
            | NodeKind::ComponentOutput(ComponentPin { component, .. }) => {
                self.components.insert(component.clone());
            }
            _ => (),
        };
        let node = GraphNodeRef {
            node: Rc::new(RefCell::new(GraphNode {
                idx: self.adjacency.len(),
                kind: node_kind,
                neighbours: HashSet::default(),
            })),
        };
        self.adjacency.push(node.clone());
        node
    }

    pub fn nodes(&self) -> Vec<GraphNodeRef<'a>> {
        self.adjacency.to_vec()
    }

    fn find_disjointed_node_sets(&self) -> Vec<usize> {
        let mut uf_component_indices: Vec<_> = (0..self.adjacency.len()).collect();
        for node_ref in &self.adjacency {
            let mut this_idx = node_ref.node.borrow().idx;
            while this_idx != uf_component_indices[this_idx] {
                this_idx = uf_component_indices[this_idx];
            }
            for neighbour in &node_ref.node.borrow().neighbours {
                let mut that_idx = neighbour.node.borrow().idx;
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

    // this should be output to input I guess
    fn output_to_input_mapping(
        &self,
        uf_component_indices: &[usize],
    ) -> HashMap<ComponentInput, HashSet<ComponentOutput>> {
        let mut output_to_input = HashMap::<ComponentOutput, HashSet<ComponentInput>>::new();

        for node_ref in &self.adjacency {
            let node = node_ref.node.borrow();
            if let NodeKind::ComponentOutput(component_output) = &node.kind {
                let node_set_idx = uf_component_indices[node.idx];
                let inputs = uf_component_indices
                    .iter()
                    .enumerate()
                    .filter(|(_, set_idx)| **set_idx == node_set_idx)
                    .filter_map(|(idx, _)| match &self.adjacency[idx].node.borrow().kind {
                        NodeKind::ComponentInput(component_input) => Some(component_input.clone()),
                        _ => None,
                    })
                    .collect::<HashSet<_>>();
                if !inputs.is_empty() {
                    output_to_input.insert(component_output.clone(), inputs);
                }
            }
        }
        output_to_input
    }

    pub fn finalize(self) -> Box<ComponentLogic> {
        let uf_component_indices = self.find_disjointed_node_sets();
        let _outputs_to_inputs = self.output_to_input_mapping(&uf_component_indices);

        println!(
            "{:?}",
            uf_component_indices
                .into_iter()
                .enumerate()
                .collect::<Vec<_>>()
        );

        unimplemented!("Convert the whole graph in a comp logic function")
    }
}

impl Debug for Graph<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for GraphNodeRef { node: node_ref } in self.adjacency.iter() {
            f.write_fmt(format_args!(
                "{:?}_{:?} -> [ ",
                node_ref.borrow().idx,
                node_ref.borrow().kind
            ))?;

            let neighbours = &mut node_ref
                .borrow()
                .neighbours
                .iter()
                .map(|neighbour| {
                    format!(
                        "{:?}_{:?}",
                        neighbour.node.borrow().idx,
                        neighbour.node.borrow().kind
                    )
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

    fn test(_: &[BitState]) -> Vec<BitState> {
        vec![]
    }

    #[test]
    fn builds_graph() {
        let comp = Rc::new(DigitalComponent::named(1, 1, &test, "test"));
        let a = NodeKind::ComponentInput(ComponentInput::new(&comp, 0));
        let b = NodeKind::Joint;
        let c = NodeKind::Joint;
        let d = NodeKind::Joint;
        let mut graph = Graph::default();
        let mut a_node = graph.add_node(a);
        let mut b_node = graph.add_node(b);
        let _c_node = graph.add_node(c);
        let mut d_node = graph.add_node(d);
        graph.add_edge(&mut a_node, &mut b_node);
        graph.add_edge(&mut d_node, &mut b_node);

        assert_eq!(
            format!("{graph:?}"),
            "\
            0_component_input(0) -> [ 1_joint]\n\
            1_joint -> [ 0_component_input(0), 3_joint]\n\
            2_joint -> [ ]\n\
            3_joint -> [ 1_joint]\n\
            "
        );
    }

    #[test]
    fn finds_disjointed_node_sets() {
        let comp = Rc::new(DigitalComponent::named(1, 1, &test, "test"));
        let a = NodeKind::ComponentInput(ComponentInput::new(&comp, 0));
        let mut graph = Graph::default();
        let mut a_node = graph.add_node(a);
        let mut b_node = graph.add_node(NodeKind::Joint);
        let _c_node = graph.add_node(NodeKind::Joint);
        let mut d_node = graph.add_node(NodeKind::Joint);
        let mut _e_node = graph.add_node(NodeKind::Joint);
        graph.add_edge(&mut a_node, &mut b_node);
        graph.add_edge(&mut d_node, &mut b_node);

        assert_eq!(graph.find_disjointed_node_sets(), vec![0, 0, 2, 0, 4]);
    }

    #[test]
    fn generates_input_to_output_mapping() {
        let reg_1 = Rc::new(DigitalComponent::named(2, 2, &test, "Reg#1"));
        let reg_2 = Rc::new(DigitalComponent::named(2, 2, &test, "Reg#2"));

        let mut graph = Graph::default();
        let _reg_1_input_0 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(&reg_1, 0)));
        let _reg_1_input_1 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(&reg_1, 1)));
        let mut reg_1_output_0 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(&reg_1, 0)));
        let mut reg_1_output_1 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(&reg_1, 1)));

        let mut joint = graph.add_node(NodeKind::Joint);

        let mut reg_2_input_0 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(&reg_2, 0)));
        let mut reg_2_input_1 =
            graph.add_node(NodeKind::ComponentInput(ComponentInput::new(&reg_2, 1)));
        let _reg_2_output_0 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(&reg_2, 0)));
        let _reg_2_output_1 =
            graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(&reg_2, 1)));

        graph.add_edge(&mut reg_1_output_0, &mut joint);
        graph.add_edge(&mut joint, &mut reg_2_input_0);
        graph.add_edge(&mut reg_1_output_1, &mut reg_2_input_1);

        let mut expected = HashMap::new();
        let mut outputs_connected_to_reg1_output_0 = HashSet::new();
        outputs_connected_to_reg1_output_0.insert(ComponentInput::new(&reg_2, 0));
        expected.insert(
            ComponentOutput::new(&reg_1, 0),
            outputs_connected_to_reg1_output_0,
        );

        let mut outputs_connected_to_reg1_output_1 = HashSet::new();
        outputs_connected_to_reg1_output_1.insert(ComponentInput::new(&reg_2, 1));
        expected.insert(
            ComponentOutput::new(&reg_1, 1),
            outputs_connected_to_reg1_output_1,
        );

        let mapping = graph.output_to_input_mapping(&graph.find_disjointed_node_sets());

        assert_eq!(mapping, expected);
    }

    #[test]
    #[should_panic]
    fn converts_graph_into_component_logic() {
        let comp = Rc::new(DigitalComponent::named(1, 1, &test, "test"));
        let a = NodeKind::ComponentInput(ComponentInput::new(&comp, 0));
        let b = NodeKind::Joint;
        let c = NodeKind::Joint;
        let d = NodeKind::Joint;
        let mut graph = Graph::default();
        let mut a_node = graph.add_node(a);
        let mut b_node = graph.add_node(b);
        let _c_node = graph.add_node(c);
        let mut d_node = graph.add_node(d);
        graph.add_edge(&mut a_node, &mut b_node);
        graph.add_edge(&mut d_node, &mut b_node);

        let _result = graph.finalize();
    }
}
