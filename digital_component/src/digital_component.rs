use std::hash::{Hash, Hasher};
use std::{fmt, ptr};

use crate::BitState;

/// Maps vector of input to vector of outputs
///
/// Return [`true`] if output values changed
pub type ComponentLogic = dyn Fn(&[BitState]) -> Vec<BitState>;

pub struct DigitalComponent<'a> {
    name: String,
    inputs: Vec<BitState>,
    outputs: Vec<BitState>,
    func: &'a ComponentLogic,
}

impl PartialEq for DigitalComponent<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        ptr::addr_eq(self, rhs)
    }
}

impl Eq for DigitalComponent<'_> {}

impl Hash for DigitalComponent<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::addr_of!(*self).hash(state);
    }
}

impl<'a> DigitalComponent<'a> {
    pub fn new(
        input_number: usize,
        output_number: usize,
        func: &'a ComponentLogic,
    ) -> DigitalComponent<'a> {
        DigitalComponent::named(input_number, output_number, func, "")
    }

    pub fn named(
        input_number: usize,
        output_number: usize,
        func: &'a ComponentLogic,
        name: &str,
    ) -> DigitalComponent<'a> {
        let mut dc = DigitalComponent {
            name: String::from(name),
            inputs: vec![BitState::Undefined; input_number],
            outputs: vec![BitState::Undefined; output_number],
            func,
        };
        dc.resolve();
        dc
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_input(&mut self, idx: usize, value: &BitState) {
        self.inputs[idx] = value.clone();
    }

    pub fn set_inputs(&mut self, values: Vec<u8>) {
        assert!(
            values.len() == self.inputs.len(),
            "Expected exactly {} inputs",
            self.inputs.len()
        );
        for (idx, input) in self.inputs.iter_mut().enumerate() {
            *input = match values[idx] {
                0 => BitState::Off,
                1 => BitState::On,
                x => panic!(
                    "Values can be comprised only of 1s and 0s. Got {} instead",
                    x
                ),
            }
        }
    }

    pub fn get_input_num(&self) -> usize {
        self.inputs.len()
    }

    pub fn get_output(&self, idx: usize) -> &BitState {
        &self.outputs[idx]
    }

    pub fn get_outputs(&self) -> Vec<u8> {
        self.outputs
            .iter()
            .map(|bit| match bit {
                BitState::On => 1,
                BitState::Off => 0,
                _ => panic!("Some outputs are in undefined state. Can't serialize to 1s and 0s"),
            })
            .collect()
    }

    pub fn get_outputs_num(&self) -> usize {
        self.outputs.len()
    }

    pub fn resolve(&mut self) -> bool {
        let new_output = (self.func)(&self.inputs);

        let changed = self.outputs.iter().ne(new_output.iter());
        self.outputs = new_output;
        changed
    }
}

impl fmt::Display for DigitalComponent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inputs: String = self
            .inputs
            .iter()
            .map(|signal| match signal {
                BitState::On => "1, ",
                BitState::Off => "0, ",
                BitState::Undefined => "u, ",
            })
            .collect();

        let outputs: String = self
            .outputs
            .iter()
            .map(|signal| match signal {
                BitState::On => "1, ",
                BitState::Off => "0, ",
                BitState::Undefined => "u, ",
            })
            .collect();
        let name = match self.name.is_empty() {
            true => "unnamed",
            false => &self.name,
        };

        write!(f, "{} inputs[{}] outputs[{}]", name, inputs, outputs)
    }
}

impl fmt::Debug for DigitalComponent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
