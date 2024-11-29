mod digital_component;
pub use digital_component::{ComponentLogic, DigitalComponent};

mod component_graph;
pub use component_graph::{ComponentInput, ComponentOutput, Graph, GraphNodeRef, NodeKind};

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum BitState {
    On,
    Off,
    Undefined,
}

#[derive(Eq, PartialEq, Hash)]
pub struct Output(pub usize);

#[derive(Eq, PartialEq, Hash)]
pub struct Input(pub usize);

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ComponentId(pub usize);
