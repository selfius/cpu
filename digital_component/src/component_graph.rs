use super::DigitalComponent;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{Debug, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::rc::Rc;

#[derive(Eq, PartialEq)]
pub enum NodeKind {
    ComponentInput {
        component: Rc<DigitalComponent>,
        input: usize,
    },
    ComponentOutput {
        component: Rc<DigitalComponent>,
        output: usize,
    },
    Input(usize),
    Output(usize),
    Joint,
}

impl Debug for NodeKind {
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

pub struct GraphNode {
    idx: usize,
    kind: NodeKind,
    neighbours: HashSet<GraphNodeRef>,
}

impl GraphNode {
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }
}

#[derive(Clone)]
pub struct GraphNodeRef {
    node: Rc<RefCell<GraphNode>>,
}

impl Debug for GraphNodeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!(
            "{:?}_{:?}",
            self.node.borrow().kind,
            ptr::addr_of!(self.node)
        ))
    }
}

impl PartialEq for GraphNodeRef {
    fn eq(&self, rhs: &Self) -> bool {
        ptr::addr_eq(&*self.node, &*rhs.node)
    }
}

impl Eq for GraphNodeRef {}

impl Hash for GraphNodeRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::addr_of!(self).hash(state);
    }
}

#[derive(PartialEq, Eq, Default)]
pub struct Graph {
    components: HashSet<Rc<DigitalComponent>>,
    adjacency: Vec<GraphNodeRef>,
}

impl Graph {
    pub fn add_edge(&mut self, a: &mut GraphNodeRef, b: &mut GraphNodeRef) {
        a.node.borrow_mut().neighbours.insert(b.clone());
        b.node.borrow_mut().neighbours.insert(a.clone());
    }

    pub fn add_node(&mut self, node_kind: NodeKind) -> GraphNodeRef {
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

    pub fn nodes(&self) -> Vec<GraphNodeRef> {
        self.adjacency.to_vec()
    }
}

impl Debug for Graph {
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

    fn test(_: &Vec<BitState>, _: &mut Vec<BitState>) -> bool {
        false
    }

    #[test]
    fn builds_graph() {
        let comp = Rc::new(DigitalComponent::named(1, 1, Box::new(test), "test"));
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

        println!("{graph:?}");

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
}
