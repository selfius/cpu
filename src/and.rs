use crate::nand::nand;
use crate::not::not;
use digital_component::{
    ComponentInput, ComponentLogic, ComponentLogicFactory, ComponentOutput, DigitalComponent,
    Graph, NodeKind,
};
use parser::parse;
use std::collections::HashMap;

pub fn and() -> Box<ComponentLogic> {
    let mut functions: HashMap<&str, Box<ComponentLogicFactory>> = HashMap::new();
    functions.insert("NAND", Box::new(|| Box::new(nand)));
    functions.insert("NOT", Box::new(not));
    parse(
        "
          ┏━━━━━━┓   ┏━━━━━┓
       ───┨      ┃   ┃     ┃
          ┃ NAND ┠───┨ NOT ┠────
       ───┨      ┃   ┃     ┃
          ┗━━━━━━┛   ┗━━━━━┛
    ",
        &functions,
    )
    .unwrap()
    .finalize()
}

/// Cascade n - 1 AND gates to have an n input AND gate.
/// For example for n = 3 an equivalent to the following circuit will be generated:
///
///     ┏━━━━━━━┓
///  ───┨       ┃
///     ┃  AND  ┠──┐  ┏━━━━━━━┓
///  ───┨       ┃  └──┨       ┃
///     ┗━━━━━━━┛     ┃  AND  ┠────
///  ─────────────────┃       ┃
///                   ┗━━━━━━━┛
///
pub fn cascade_and(n: usize) -> Box<ComponentLogicFactory> {
    Box::new(move || -> Box<ComponentLogic> {
        let mut graph = Graph::default();
        let and_gates = (0..n - 1)
            .map(|_| graph.add_component(DigitalComponent::new(2, 1, and())))
            .collect::<Vec<_>>();

        let outer_inputs = (0..n)
            .map(|input_idx| graph.add_node(NodeKind::Input(input_idx)))
            .collect::<Vec<_>>();

        let outer_output = graph.add_node(NodeKind::Output(0));

        let and_gates_inputs = and_gates
            .iter()
            .map(|and_gate| {
                (
                    graph.add_node(NodeKind::ComponentInput(ComponentInput::new(*and_gate, 0))),
                    graph.add_node(NodeKind::ComponentInput(ComponentInput::new(*and_gate, 1))),
                )
            })
            .collect::<Vec<_>>();

        let and_gates_outputs = and_gates
            .iter()
            .map(|and_gate| {
                graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(
                    *and_gate, 0,
                )))
            })
            .collect::<Vec<_>>();

        graph.add_edge(&outer_inputs[0], &and_gates_inputs[0].0);

        for (outer_input, and_gate_inputs) in outer_inputs[1..n].iter().zip(and_gates_inputs.iter())
        {
            graph.add_edge(outer_input, &and_gate_inputs.1);
        }

        for (and_gate_output, next_and_gate_inputs) in and_gates_outputs
            .iter()
            .zip(and_gates_inputs.iter().skip(1))
        {
            graph.add_edge(and_gate_output, &next_and_gate_inputs.0);
        }

        graph.add_edge(&outer_output, and_gates_outputs.iter().last().unwrap());

        graph.finalize()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use digital_component::BitState::*;

    #[test]
    fn ands() {
        let mut and_gate = and();

        let mut output = vec![Undefined];

        and_gate(&[Off, Undefined], &mut output);
        assert_eq!(output, vec![Off]);

        and_gate(&[Off, Off], &mut output);
        assert_eq!(output, vec![Off]);

        and_gate(&[On, Off], &mut output);
        assert_eq!(output, vec![Off]);

        and_gate(&[On, Off], &mut output);
        assert_eq!(output, vec![Off]);

        and_gate(&[On, On], &mut output);
        assert_eq!(output, vec![On]);
    }

    #[test]
    fn cascades_and_gates() {
        let mut and_gate = cascade_and(3)();

        let mut output = vec![Undefined];

        and_gate(&[Off, Off, Undefined], &mut output);
        assert_eq!(output, vec![Off]);

        and_gate(&[Off, On, Undefined], &mut output);
        assert_eq!(output, vec![Off]);

        and_gate(&[On, On, On], &mut output);
        assert_eq!(output, vec![On]);
    }
}
