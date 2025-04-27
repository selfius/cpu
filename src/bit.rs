use crate::nand::nand;
use digital_component::{ComponentLogic, ComponentLogicFactory};
use parser::parse;
use std::collections::HashMap;

pub fn bit() -> Box<ComponentLogic> {
    let mut functions: HashMap<&str, Box<ComponentLogicFactory>> = HashMap::new();
    functions.insert("NAND", Box::new(|| Box::new(nand)));
    parse(
       "
             ┏━━━━━━┓                    ┏━━━━━━┓
         ────┨      ┠─┬──────────────────┨      ┃
             ┃ NAND ┃ │                  ┃ NAND ┠───┬───
           ┌─┨      ┃ │                ┌─┨      ┃   │
           │ ┗━━━━━━┛ │                │ ┗━━━━━━┛   │
           │          │                └──────────┐ │
           │          │                           │ │
           │          │ ┏━━━━━━┓         ┏━━━━━━┓ │ │
           │          └─┨      ┠─────────┨      ┃ │ │
           │            ┃ NAND ┃         ┃ NAND ┠─┘ │
         ──┴────────────┨      ┃       ┌─┨      ┃   │
                        ┗━━━━━━┛       │ ┗━━━━━━┛   │
                                       └────────────┘
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
    fn bit_bits() {
        let mut bit_logic = bit();

        let mut output = vec![BitState::Undefined];

        bit_logic(&[BitState::Off, BitState::Off], &mut output);
        println!("=============");

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
