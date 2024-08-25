use std::ops::Fn;

fn main() {
    let mut nand_component = DigitalComponent::new(2, 1, &nand);
    assert_eq!(nand_component.get_output(0), true);
    nand_component.set_input(0, true);
    assert_eq!(nand_component.resolve(), false);
    assert_eq!(nand_component.get_output(0), true);

    nand_component.set_input(1, true);
    assert_eq!(nand_component.resolve(), true);
    assert_eq!(nand_component.get_output(0), false);
}

struct DigitalComponent<'a> {
    inputs: Vec<bool>,
    outputs: Vec<bool>,
    func: &'a ComponentLogic,
}

fn nand(input: &Vec<bool>, output: &mut Vec<bool>) -> bool {
    assert!(input.len() == 2, "NAND gate must have exactly two inputs");
    assert!(output.len() == 1, "NAND gate must have exactly one ouput");
    let previous_value = output[0];
    output[0] = match (input[0], input[1]) {
        (true, true) => false,
        _ => true,
    };
    previous_value != output[0]
}

/// Maps vector of input to vector of outputs
///
/// Return [`true`] if output values changed
type ComponentLogic = dyn Fn(&Vec<bool>, &mut Vec<bool>) -> bool;

impl<'a> DigitalComponent<'a> {
    fn new(input_number: usize, output_number: usize, func: &'a ComponentLogic) -> Self {
        let mut dc = DigitalComponent {
            inputs: vec![false; input_number],
            outputs: vec![false; output_number],
            func,
        };
        dc.resolve();
        dc
    }

    fn set_input(&mut self, idx: usize, value: bool) {
        self.inputs[idx] = value;
    }

    fn get_output(&self, idx: usize) -> bool {
        self.outputs[idx]
    }

    fn resolve(&mut self) -> bool {
        (*self.func)(&self.inputs, &mut self.outputs)
    }
}
