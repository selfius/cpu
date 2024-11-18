use super::{ComponentLogic, DigitalComponent};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{Debug, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::rc::Rc;

#[derive(Eq, PartialEq)]
pub enum NodeKind<'a> {
    ComponentInput {
        component: Rc<DigitalComponent<'a>>,
        input: usize,
    },
    ComponentOutput {
        component: Rc<DigitalComponent<'a>>,
        output: usize,
    },
    Input(usize),
    Output(usize),
    Joint,
}

impl Debug for NodeKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!(
            "{}",
            match self {
                NodeKind::ComponentInput { input, .. } => format!("component_input({})", input),
                NodeKind::ComponentOutput { output, .. } => format!("component_output({})", output),
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
            NodeKind::ComponentInput { component, .. }
            | NodeKind::ComponentOutput { component, .. } => {
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

    fn _finalize(self) -> Box<ComponentLogic> {
        // construct the union find structure
        // to simplify the connections and create a new adjacency graph out of that
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
        let a = NodeKind::ComponentInput {
            component: comp.clone(),
            input: 0,
        };
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
    #[should_panic]
    fn converts_graph_into_component_logic() {
        let comp = Rc::new(DigitalComponent::named(1, 1, &test, "test"));
        let a = NodeKind::ComponentInput {
            component: comp.clone(),
            input: 0,
        };
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

        let _result = graph._finalize();
    }
}
