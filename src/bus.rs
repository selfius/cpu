use crate::{BitState, DigitalComponent};

struct Bus {
    dc: DigitalComponent,
}

impl Bus {
    fn new(lanes_num: usize) -> Bus {
        Bus {
            dc: DigitalComponent::new(lanes_num, lanes_num, Box::new(pass_through)),
        }
    }
}

fn pass_through(input: &Vec<BitState>, output: &mut Vec<BitState>) -> bool {
    for (input_lane, output_lane) in input.iter().zip(output.iter_mut()) {
        *output_lane = input_lane.clone();
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bass_pass_through() {
        let Bus { dc: mut bus } = Bus::new(2);
        bus.set_inputs(vec![0, 0]);
        bus.resolve();
        assert_eq!(bus.get_outputs(), vec![0, 0]);

        bus.set_inputs(vec![1, 0]);
        bus.resolve();
        assert_eq!(bus.get_outputs(), vec![1, 0]);
    }
}
