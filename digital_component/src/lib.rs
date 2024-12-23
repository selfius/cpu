mod digital_component;
pub use digital_component::{ComponentLogic, ComponentLogicFactory, DigitalComponent};

mod component_graph;
pub use component_graph::{ComponentInput, ComponentOutput, Graph, GraphNodeRef, NodeKind};

mod debug_logger;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash, Default)]
pub enum BitState {
    On,
    Off,
    #[default]
    Undefined,
}

#[derive(Eq, PartialEq, Hash)]
pub struct Output(pub usize);

#[derive(Eq, PartialEq, Hash)]
pub struct Input(pub usize);

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ComponentId(pub usize);
