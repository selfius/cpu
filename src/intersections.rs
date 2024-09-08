use crate::{BitState, DigitalComponent};

fn intersection(
    name: &str,
    in_lane_width: usize,
    out_lane_width: usize,
    resolver: fn(&Vec<BitState>, &mut Vec<BitState>) -> bool,
) -> DigitalComponent {
    DigitalComponent::named(in_lane_width, out_lane_width, Box::new(resolver), name)
}

pub fn split(name: Option<&str>, lane_width: usize) -> DigitalComponent {
    intersection(
        name.unwrap_or("join"),
        lane_width,
        lane_width * 2,
        resolve_split,
    )
}

fn resolve_split(inputs: &Vec<BitState>, outputs: &mut Vec<BitState>) -> bool {
    let lane_width = inputs.len();
    let mut changed = false;
    let (output_1, output_2) = outputs.split_at_mut(lane_width);
    for ((o_1, o_2), i) in output_1.iter_mut().zip(output_2).zip(inputs) {
        if o_1 != i {
            *o_1 = i.clone();
            *o_2 = i.clone();
            changed = true;
        }
    }
    changed
}

pub fn join(name: Option<&str>, lane_width: usize) -> DigitalComponent {
    intersection(
        name.unwrap_or("join"),
        lane_width * 2,
        lane_width,
        resolve_join,
    )
}

fn resolve_join(inputs: &Vec<BitState>, outputs: &mut Vec<BitState>) -> bool {
    let lane_width = outputs.len();
    let mut changed = false;
    let (input_1, input_2) = inputs.split_at(lane_width);
    for ((i_1, i_2), o) in input_1.iter().zip(input_2).zip(outputs.iter_mut()) {
        let old_value = o.clone();
        let new_value = match (i_1, i_2) {
            (BitState::On, _) | (_, BitState::On) => BitState::On,
            (BitState::Undefined, other) | (other, BitState::Undefined) => other.clone(),
            _ => BitState::Off,
        };
        changed = changed || old_value != new_value;
        *o = new_value;
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::{
        Register, ENABLE_PIN_IDX, IO_PINS_NUMBER as BUS_WIDTH, SET_VALUE_IN_IDX,
    };
    use crate::{composite_component_logic, BitState, ComponentGraph, ComponentId};

    #[test]
    fn splits_input_in_two() {
        let mut splitter = split(None, 2);
        splitter.set_inputs(vec![0, 1]);
        splitter.resolve();

        let outputs_combined = splitter.get_outputs();
        let (output_1, output_2) = outputs_combined.split_at(2);

        assert_eq!(output_1, vec![0, 1]);
        assert_eq!(output_2, vec![0, 1]);
    }

    #[test]
    fn joins_two_inputs_together() {
        let mut joinner = join(None, 2);
        joinner.set_inputs(vec![0, 1, 1, 0]);
        joinner.resolve();
        assert_eq!(joinner.get_outputs(), vec![1, 1]);

        joinner.set_inputs(vec![1, 0, 1, 0]);
        joinner.resolve();
        assert_eq!(joinner.get_outputs(), vec![1, 0]);
    }

    // Circuit schematics for the next test:
    //
    //           a             b   c             d
    //     0..7 ═╦═════════════╦═══╦═════════════╦═
    //           ║  ┌───────┐  ║   ║  ┌───────┐  ║
    //           ╚══│I     O│══╝   ╚══│I     O│══╝
    //         8 ───│E R#1  │   10 ───│E R#2  │
    //         9 ───│S      │   11 ───│S      │
    //              └───────┘         └───────┘
    #[test]
    fn can_be_used_to_connect_registers_to_bus() {
        let components = vec![
            split(Some("a"), BUS_WIDTH),
            join(Some("b"), BUS_WIDTH),
            split(Some("c"), BUS_WIDTH),
            join(Some("d"), BUS_WIDTH),
            Register::named("Register #1").dc,
            Register::named("Register #2").dc,
        ];
        let mut cg = ComponentGraph::new(components);
        // a to R#1
        cg.connect_range(
            (ComponentId(0), 0..BUS_WIDTH),
            (ComponentId(4), 0..BUS_WIDTH),
        );
        // a to b
        cg.connect_range(
            (ComponentId(0), BUS_WIDTH..BUS_WIDTH * 2),
            (ComponentId(1), 0..BUS_WIDTH),
        );
        // R#1 to b
        cg.connect_range(
            (ComponentId(4), 0..BUS_WIDTH),
            (ComponentId(1), BUS_WIDTH..BUS_WIDTH * 2),
        );
        // b to c
        cg.connect_range(
            (ComponentId(1), 0..BUS_WIDTH),
            (ComponentId(2), 0..BUS_WIDTH),
        );
        // c to R#2
        cg.connect_range(
            (ComponentId(2), 0..BUS_WIDTH),
            (ComponentId(5), 0..BUS_WIDTH),
        );
        // c to d
        cg.connect_range(
            (ComponentId(2), BUS_WIDTH..BUS_WIDTH * 2),
            (ComponentId(3), 0..BUS_WIDTH),
        );
        // R#2 to d
        cg.connect_range(
            (ComponentId(5), 0..BUS_WIDTH),
            (ComponentId(3), BUS_WIDTH..BUS_WIDTH * 2),
        );

        fn map_inputs(inputs: &Vec<BitState>, components: &mut Vec<DigitalComponent>) {
            // 0..7 bus io pins
            // 8, 9 enable, set bits for the first register
            // 10, 11 - second register
            for io_pin in 0..BUS_WIDTH {
                components[0].set_input(io_pin, &inputs[io_pin]);
            }
            components[4].set_input(ENABLE_PIN_IDX, &inputs[8]);
            components[4].set_input(SET_VALUE_IN_IDX, &inputs[9]);

            components[5].set_input(ENABLE_PIN_IDX, &inputs[10]);
            components[5].set_input(SET_VALUE_IN_IDX, &inputs[11]);
        }

        fn map_outputs(outputs: &mut Vec<BitState>, components: &mut Vec<DigitalComponent>) {
            for io_pin in 0..BUS_WIDTH {
                outputs[io_pin] = components[3].get_output(io_pin).clone();
            }
        }

        let mut test_bed = DigitalComponent::new(
            12,
            8,
            Box::new(composite_component_logic(
                cg,
                Box::new(map_inputs),
                Box::new(map_outputs),
            )),
        );

        // init registers with zero's
        test_bed.set_inputs(vec![0; 12]);
        test_bed.set_input(9, &BitState::On);
        test_bed.set_input(11, &BitState::On);
        test_bed.resolve();
        assert_eq!(test_bed.get_outputs(), vec![0; 8]);

        //let put 42 into the first register
        test_bed.set_inputs(vec![
            /* io pins */ 0, 1, 0, 0, 0, 0, 1, 0, /* control pins */ 0, 1, 0, 0,
        ]);
        test_bed.resolve();

        // enable R#1 again
        test_bed.set_inputs(vec![
            /* io pins */ 0, 0, 0, 0, 0, 0, 0, 0, /* control pins */ 1, 0, 0, 0,
        ]);
        test_bed.resolve();
        assert_eq!(test_bed.get_outputs(), vec![0, 1, 0, 0, 0, 0, 1, 0]);

        // check that R#2 is empty
        test_bed.set_inputs(vec![
            /* io pins */ 0, 0, 0, 0, 0, 0, 0, 0, /* control pins */ 0, 0, 1, 0,
        ]);
        test_bed.resolve();
        assert_eq!(test_bed.get_outputs(), vec![0; 8]);

        // enable R#1 and turn on the set bit of R#2
        test_bed.set_inputs(vec![
            /* io pins */ 0, 0, 0, 0, 0, 0, 0, 0, /* control pins */ 1, 0, 0, 1,
        ]);
        test_bed.resolve();

        // R#2 should have hold 42 now
        test_bed.set_inputs(vec![
            /* io pins */ 0, 0, 0, 0, 0, 0, 0, 0, /* control pins */ 0, 0, 1, 0,
        ]);
        test_bed.resolve();
        assert_eq!(test_bed.get_outputs(), vec![0, 1, 0, 0, 0, 0, 1, 0]);
    }
}
