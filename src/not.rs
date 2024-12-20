use crate::nand::nand;
use digital_component::{ComponentLogic, ComponentLogicFactory};
use parser::parse;
use std::collections::HashMap;

pub fn not() -> Box<ComponentLogic> {
    let mut functions: HashMap<&str, Box<ComponentLogicFactory>> = HashMap::new();
    functions.insert("nand", Box::new(|| Box::new(nand)));
    parse(
        "
           ┏━━━━┓
        ─┬─┨nand┠──
         └─┨    ┃
           ┗━━━━┛
    ",
        &functions,
    )
    .unwrap()
    .finalize()
}


#[cfg(test)]
mod tests {
    use super::*;
    use digital_component::*;

    #[test]
    fn nots() {
        let mut not_gate = not();

        let mut output = vec![BitState::Undefined];

        not_gate(&[BitState::Off], &mut output);
        assert_eq!(output, vec![BitState::On]);

        not_gate(&[BitState::On], &mut output);
        assert_eq!(output, vec![BitState::On]);
    }
}
