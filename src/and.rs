use crate::nand::nand;
use crate::not::not;
use digital_component::{ComponentLogic, ComponentLogicFactory};
use parser::parse;
use std::collections::HashMap;

pub fn and() -> Box<ComponentLogic> {
    let mut functions: HashMap<&str, Box<ComponentLogicFactory>> = HashMap::new();
    functions.insert("nand", Box::new(|| Box::new(nand)));
    functions.insert("not", Box::new(not));
    parse(
        "
        ┏━━━━┓ ┏━━━┓
       ─┨nand┠─┨not┠─
       ─┨    ┃ ┗━━━┛
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
    fn ands() {
        let mut and_gate = and();

        let mut output = vec![BitState::Undefined];

        and_gate(&[BitState::Off, BitState::Undefined], &mut output);
        assert_eq!(output, vec![BitState::Off]);

        and_gate(&[BitState::Off, BitState::Off], &mut output);
        assert_eq!(output, vec![BitState::Off]);

        and_gate(&[BitState::On, BitState::Off], &mut output);
        assert_eq!(output, vec![BitState::Off]);

        and_gate(&[BitState::On, BitState::Off], &mut output);
        assert_eq!(output, vec![BitState::Off]);

        and_gate(&[BitState::On, BitState::On], &mut output);
        assert_eq!(output, vec![BitState::On]);
    }
}
