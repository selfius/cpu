use crate::nand::nand;
use digital_component::{ComponentLogic, ComponentLogicFactory};
use parser::parse;
use std::collections::HashMap;

pub fn bit() -> Box<ComponentLogic> {
    let mut functions: HashMap<&str, Box<ComponentLogicFactory>> = HashMap::new();
    functions.insert("nand", Box::new(|| Box::new(nand)));
    parse(
        "
           ┏━━━━┓                ┏━━━━┓
        ───┨nand┠────┬───────────┨nand┃
         ┌─┨    ┃    │          ┌┨    ┠┬──
         │ ┗━━━━┛    │          │┗━━━━┛│
         │           │          └─────┐│
         │           │        ┌───────┼┘
         │           │ ┏━━━━┓ │ ┏━━━━┓│
         │           └─┨nand┃ └─┨nand┠┘
        ─┴─────────────┨    ┠───┨    ┃
                       ┗━━━━┛   ┗━━━━┛
    ",
        //            ┏━━━━┓                ┏━━━━┓
        //       12───0 0  2────┬───────────3 1  ┃
        //          ┌─1    ┃    │15        ┌4    5┬──13
        //          │ ┗━━━━┛    │          │┗━━━━┛│16
        //          │           │          └─────┐│
        //          │           │        ┌───────┼┘
        //          │           │ ┏━━━━┓ │ ┏━━━━┓│
        //          │17         └─6 2  ┃ └─9 3 11┘
        //       14─┴─────────────7    8───10   ┃
        //                        ┗━━━━┛   ┗━━━━┛
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
    fn bit_bits() {
        let mut bit_logic = bit();

        let mut output = vec![BitState::Undefined];

        bit_logic(&[BitState::Off, BitState::Off], &mut output);

        bit_logic(&[BitState::Off, BitState::On], &mut output);
        assert_eq!(output, vec![BitState::Off]);

        bit_logic(&[BitState::Off, BitState::Off], &mut output);
        assert_eq!(output, vec![BitState::Off]);

        bit_logic(&[BitState::On, BitState::On], &mut output);
        assert_eq!(output, vec![BitState::On]);

        bit_logic(&[BitState::Off, BitState::Off], &mut output);
        assert_eq!(output, vec![BitState::On]);
    }
}
