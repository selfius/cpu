use crate::and::{and, cascade_and};
use crate::not::not;
use digital_component::{
    ComponentInput, ComponentLogic, ComponentLogicFactory, ComponentOutput, DigitalComponent,
    Graph, NodeKind,
};
use parser::parse;
use std::collections::HashMap;

pub fn decoder_2_to_4() -> Box<ComponentLogic> {
    let mut functions: HashMap<&str, Box<ComponentLogicFactory>> = HashMap::new();
    functions.insert("not", Box::new(not));
    functions.insert("and", Box::new(and));
    parse(
        "
         ┏━━━┓       ┏━━━┓
      ──┬┨not┠─────┬─┨   ┃
        │┗━━━┛┏━━━┓│ ┃and┠─
      ──┼┬────┨not┠┼┬┨   ┃
        ││    ┗━━━┛││┗━━━┛
        ││         ││┏━━━┓
        ││         └┼┨and┠─
        │├──────────┼┨   ┃
        ││          │┗━━━┛
        ││          │┏━━━┓
        ├┼──────────┼┨and┠─
        ││          └┨   ┃
        ││           ┗━━━┛
        ││           ┏━━━┓
        └┼───────────┨and┠─
         └───────────┨   ┃
                     ┗━━━┛
    ",
        &functions,
    )
    .unwrap()
    .finalize()
}

pub fn decoder(n: usize) -> Box<ComponentLogicFactory> {
    Box::new(move || -> Box<ComponentLogic> {
        let mut graph = Graph::default();
        let not_gates = (0..n)
            .map(|_| graph.add_component(DigitalComponent::new(1, 1, not())))
            .collect::<Vec<_>>();

        let outer_inputs = (0..n)
            .map(|input_idx| graph.add_node(NodeKind::Input(input_idx)))
            .collect::<Vec<_>>();

        let not_gates_inputs = not_gates
            .iter()
            .map(|not_gate| {
                graph.add_node(NodeKind::ComponentInput(ComponentInput::new(*not_gate, 0)))
            })
            .collect::<Vec<_>>();

        for (input, not_gate) in outer_inputs.iter().zip(not_gates_inputs.iter()) {
            graph.add_edge(input, not_gate);
        }

        let not_gates_outputs = not_gates
            .iter()
            .map(|not_gate| {
                graph.add_node(NodeKind::ComponentOutput(ComponentOutput::new(
                    *not_gate, 0,
                )))
            })
            .collect::<Vec<_>>();

        let number_of_outputs = 2_u32.pow(n as u32);
        let and_gates = (0..number_of_outputs)
            .map(|_| graph.add_component(DigitalComponent::new(n, 1, cascade_and(n)())))
            .collect::<Vec<_>>();

        let and_gates_inputs = and_gates
            .iter()
            .map(|and_gate| {
                (0..n)
                    .map(|input_idx| {
                        graph.add_node(NodeKind::ComponentInput(ComponentInput::new(
                            *and_gate, input_idx,
                        )))
                    })
                    .collect::<Vec<_>>()
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

        let outer_outputs = (0..number_of_outputs as usize)
            .map(|output_idx| graph.add_node(NodeKind::Output(output_idx)))
            .collect::<Vec<_>>();

        for (outer_output, and_gate_output) in outer_outputs.iter().zip(and_gates_outputs.iter()) {
            graph.add_edge(outer_output, and_gate_output);
        }

        for (output_idx, and_gate_inputs) in and_gates_inputs.iter().enumerate() {
            let mut bits = output_idx;
            for bit_idx in (0..n).rev() {
                if bits % 2 == 0 {
                    graph.add_edge(&not_gates_outputs[bit_idx], &and_gate_inputs[bit_idx]);
                } else {
                    graph.add_edge(&outer_inputs[bit_idx], &and_gate_inputs[bit_idx]);
                }
                bits >>= 1;
            }
        }
        graph.finalize()
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use digital_component::BitState::*;

    #[test]
    fn decoder_parsed_from_diagram() {
        let mut decoder = decoder_2_to_4();

        let mut output = vec![Undefined; 4];
        let input = [Off, Off];
        decoder(&input, &mut output);
        assert_eq!(output, vec![On, Off, Off, Off]);

        let input = [Off, On];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, On, Off, Off]);

        let input = [On, Off];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, Off, On, Off]);

        let input = [On, On];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, Off, Off, On]);
    }

    #[test]
    fn decoder_generated_with_code() {
        let mut decoder = decoder(2)();

        let mut output = vec![Undefined; 4];
        let input = [Off, Off];
        decoder(&input, &mut output);
        assert_eq!(output, vec![On, Off, Off, Off]);

        let input = [Off, On];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, On, Off, Off]);

        let input = [On, Off];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, Off, On, Off]);

        let input = [On, On];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, Off, Off, On]);
    }

    #[test]
    fn decoder_generate_with_more_inputs() {
        let mut decoder = decoder(3)();

        let mut output = vec![Undefined; 8];
        let input = [Off, Off, Off];
        decoder(&input, &mut output);
        assert_eq!(output, vec![On, Off, Off, Off, Off, Off, Off, Off,]);

        let input = [On, Off, On];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, Off, Off, Off, Off, On, Off, Off,]);

        let input = [On, On, On];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, Off, Off, Off, Off, Off, Off, On]);
    }
}
