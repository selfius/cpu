use crate::{and::cascade_and as and, intersections::split, not::not};
use crate::{
    composite_component_logic, BitState, ComponentGraph, ComponentId, DigitalComponent, Input,
    Output, OutputMappingFunc,
};

//       ┌───┐
// I1 ─0─┤NOT├─────┐
//     │ └───┘     │
//     │   ┌───┐   │
// I2 ─┼─1─┤NOT├───┼─┐
//     │ │ └───┘   │ │
//     │ │   ┌───┐ │ │
// I3 ─┼─┼─2─┤NOT├─┼─┼─┐
//     │ │ │ └───┘ │ │ │ ┌─────┐
//     │ │ │       3─┼─┼─┤     │
//     │ │ │       │ 4─┼─┤ AND ├─ O1 (0 0 0)
//     │ │ │       │ │ 5─┤     │
//     │ │ │       │ │ │ └─────┘
//     │ │ │       │ │ │ ┌─────┐
//     │ │ │       6─┼─┼─┤     │
//     │ │ │       │ 7─┼─┤ AND ├─ O2 (0 0 1)
//     │ │ 8───────┼─┼─┼─┤     │
//     │ │ │       │ │ │ └─────┘
//
//     ┊ ┊ ┊       ┊ ┊ ┊
//
//     │ │ │             ┌─────┐
//     └─┼─┼─────────────┤     │
//       └─┼─────────────┤ AND ├─ O8 (1 1 1)
//         └─────────────┤     │
//                       └─────┘

pub fn decoder(name: &str, in_lanes_num: usize) -> DigitalComponent {
    let out_lanes_num = 2_usize.pow(in_lanes_num as u32);
    let number_of_splits = in_lanes_num * out_lanes_num + in_lanes_num;
    let first_not_gate_idx = number_of_splits;
    let mut components = ComponentGraph::new(
        // 3 unique intersections for every AND gate/output
        (0..number_of_splits)
            .map(|num| split(Some(&format!("dec_split#{num}")), 1))
            .chain(
                // one not gate per input
                (0..in_lanes_num).map(|num| not(&format!("dec_not#{num}"))),
            )
            // one AND per output
            .chain((0..out_lanes_num).map(|num| and(&format!("dec_and#{num}"), in_lanes_num)))
            .collect(),
    );

    for (not_gate_idx, split_idx) in
        (first_not_gate_idx..first_not_gate_idx + in_lanes_num).zip(0..in_lanes_num)
    {
        components.connect(
            (ComponentId(split_idx), Output(0)),
            (ComponentId(not_gate_idx), Input(0)),
        );
    }

    let first_and_gate_idx = first_not_gate_idx + in_lanes_num;

    let mut last_before_not = (0..in_lanes_num).rev().collect::<Vec<_>>();
    let mut last_after_not = last_before_not
        .iter()
        .map(|num| num + first_not_gate_idx)
        .collect();
    let mut last_split_created = in_lanes_num - 1;

    // Now we have NOT gates wired, so for every output we need to wire AND to correct lanes.
    for output_idx in first_and_gate_idx..(first_and_gate_idx + out_lanes_num) {
        let mut idx = output_idx - first_and_gate_idx;
        for and_gate_lane in 0..in_lanes_num {
            let last_splitter_array = match idx & 1 {
                1 => &mut last_after_not,
                0 => &mut last_before_not,
                _ => panic!("nope"),
            };
            components.connect(
                (ComponentId(last_splitter_array[and_gate_lane]), Output(0)),
                (ComponentId(last_split_created + 1), Input(0)),
            );
            last_split_created += 1;
            last_splitter_array[and_gate_lane] = last_split_created;
            components.connect(
                (ComponentId(last_split_created), Output(1)),
                (ComponentId(output_idx), Input(and_gate_lane)),
            );
            idx >>= 1;
        }
    }

    DigitalComponent::named(
        in_lanes_num,
        out_lanes_num,
        Box::new(composite_component_logic(
            components,
            Box::new(map_inputs),
            Box::new(map_outputs(first_and_gate_idx)),
        )),
        name,
    )
}

fn map_inputs(inputs: &Vec<BitState>, components: &mut Vec<DigitalComponent>) {
    for (input, component) in inputs.iter().zip(components) {
        component.set_input(0, input);
    }
}

fn map_outputs(first_and_gate_index: usize) -> Box<OutputMappingFunc> {
    Box::new(
        move |outputs: &mut Vec<BitState>, components: &mut Vec<DigitalComponent>| {
            for (component, output) in components[first_and_gate_index..].iter().zip(outputs) {
                *output = component.get_output(0).clone();
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_output_per_variant() {
        let mut decoder = decoder("test", 2);
        decoder.set_inputs(vec![0, 0]);
        decoder.resolve();
        assert_eq!(decoder.get_outputs(), vec![0, 0, 0, 1]);

        decoder.set_inputs(vec![0, 1]);
        decoder.resolve();
        assert_eq!(decoder.get_outputs(), vec![0, 0, 1, 0]);

        decoder.set_inputs(vec![1, 0]);
        decoder.resolve();
        assert_eq!(decoder.get_outputs(), vec![0, 1, 0, 0]);

        decoder.set_inputs(vec![1, 1]);
        decoder.resolve();
        assert_eq!(decoder.get_outputs(), vec![1, 0, 0, 0]);
    }

    #[test]
    fn check_hard_coded_logic_for_two_lanes() {
        let mut decoder = decoder("test", 3);
        decoder.set_inputs(vec![0, 0, 0]);
        decoder.resolve();
        assert_eq!(decoder.get_outputs(), vec![0, 0, 0, 0, 0, 0, 0, 1]);

        decoder.set_inputs(vec![0, 1, 1]);
        decoder.resolve();
        assert_eq!(decoder.get_outputs(), vec![0, 0, 0, 0, 1, 0, 0, 0]);
    }
}
