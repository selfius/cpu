use std::hash::{Hash, Hasher};
use std::{fmt, ptr};

use crate::BitState;

/// Maps vector of input to vector of outputs
///
/// Return [`true`] if output values changed
pub type ComponentLogic = dyn FnMut(&[BitState], &mut [BitState]);

pub type ComponentLogicFactory = dyn Fn() -> Box<ComponentLogic>;

pub struct DigitalComponent {
    input_num: usize,
    output_num: usize,
    func: Box<ComponentLogic>,
}

impl PartialEq for DigitalComponent {
    fn eq(&self, rhs: &Self) -> bool {
        ptr::addr_eq(self, rhs)
    }
}

impl Eq for DigitalComponent {}

impl Hash for DigitalComponent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::addr_of!(*self).hash(state);
    }
}

impl DigitalComponent {
    pub fn new(input_num: usize, output_num: usize, func: Box<ComponentLogic>) -> DigitalComponent {
        DigitalComponent {
            input_num,
            output_num,
            func,
        }
    }

    pub fn get_input_num(&self) -> usize {
        self.input_num
    }

    pub fn get_output_num(&self) -> usize {
        self.output_num
    }

    pub fn get_func(&mut self) -> &mut ComponentLogic {
        &mut self.func
    }
}

impl fmt::Display for DigitalComponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "[ {} -> {} ]",
            self.input_num, self.output_num
        ))
    }
}

impl fmt::Debug for DigitalComponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
