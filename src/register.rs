use crate::and::and;
use crate::bit::bit;
use digital_component::{ComponentLogic, ComponentLogicFactory};
use parser::parse;
use std::collections::HashMap;

pub fn register() -> Box<ComponentLogic> {
    let mut functions: HashMap<&str, Box<ComponentLogicFactory>> = HashMap::new();
    functions.insert("bit", Box::new(bit));
    functions.insert("and", Box::new(and));
    parse(
        "
        ┏━━━┓  ┏━━━┓
     ───┨bit┠──┨and┠─
      ┌─┨   ┃ ┌┨   ┃
      │ ┗━━━┛ │┗━━━┛
      │ ┏━━━┓ │┏━━━┓
     ─┼─┨bit┠─┼┨and┠─
      ├─┨   ┃ ├┨   ┃
      │ ┗━━━┛ │┗━━━┛
      │ ┏━━━┓ │┏━━━┓
     ─┼─┨bit┠─┼┨and┠─
      ├─┨   ┃ ├┨   ┃
      │ ┗━━━┛ │┗━━━┛
      │ ┏━━━┓ │┏━━━┓
     ─┼─┨bit┠─┼┨and┠─
      ├─┨   ┃ ├┨   ┃
      │ ┗━━━┛ │┗━━━┛
      │ ┏━━━┓ │┏━━━┓
     ─┼─┨bit┠─┼┨and┠─
      ├─┨   ┃ ├┨   ┃
      │ ┗━━━┛ │┗━━━┛
      │ ┏━━━┓ │┏━━━┓
     ─┼─┨bit┠─┼┨and┠─
      ├─┨   ┃ ├┨   ┃
      │ ┗━━━┛ │┗━━━┛
      │ ┏━━━┓ │┏━━━┓
     ─┼─┨bit┠─┼┨and┠─
      ├─┨   ┃ ├┨   ┃
      │ ┗━━━┛ │┗━━━┛
      │ ┏━━━┓ │┏━━━┓
     ─┼─┨bit┠─┼┨and┠─
      ├─┨   ┃ ├┨   ┃
      │ ┗━━━┛ │┗━━━┛
     ─┘       │
     ─────────┘
    ",
        &functions,
    )
    .unwrap()
    .finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use digital_component::BitState::*;

    pub const SET_BIT: usize = 8;
    pub const ENABLE_BIT: usize = 9;

    #[test]
    fn stores_and_outputs_values() {
        let mut reg = register();

        let mut output = vec![Undefined; 8];
        let mut input = [Off; 10];
        input[SET_BIT] = Off;

        reg(&input, &mut output);
        assert_eq!(output, vec![Off; 8]);

        input[ENABLE_BIT] = On;
        assert_eq!(output, vec![Off; 8]);

        input = [
            Off, On, On, Off, On, On, Off, Off, /*set bit*/ On, /*enable bit*/ Off,
        ];
        reg(&input, &mut output);
        assert_eq!(output, vec![Off; 8]);

        input = [Off; 10];
        input[ENABLE_BIT] = On;

        reg(&input, &mut output);
        assert_eq!(output, vec![Off, On, On, Off, On, On, Off, Off]);
    }
}
