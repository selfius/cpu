use crate::and::and;
use crate::not::not;
use digital_component::{ComponentLogic, ComponentLogicFactory};
use parser::parse;
use std::collections::HashMap;

pub fn decoder_2_to_4() -> Box<ComponentLogic> {
    let mut functions: HashMap<&str, Box<ComponentLogicFactory>> = HashMap::new();
    functions.insert("not", Box::new(not));
    functions.insert("and", Box::new(and));
    parse(
        "
         ┏━━━┓       ┏━━━┓
      ──┬┨not┠─────┬─┨   ┃
        │┗━━━┛┏━━━┓│ ┃and┠─
      ──┼┬────┨not┠┼┬┨   ┃
        ││    ┗━━━┛││┗━━━┛
        ││         ││┏━━━┓
        ││         └┼┨and┠─
        │├──────────┼┨   ┃
        ││          │┗━━━┛
        ││          │┏━━━┓
        ├┼──────────┼┨and┠─
        ││          └┨   ┃
        ││           ┗━━━┛
        ││           ┏━━━┓
        └┼───────────┨and┠─
         └───────────┨   ┃
                     ┗━━━┛
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

    #[test]
    fn decodes() {
        let mut decoder = decoder_2_to_4();

        let mut output = vec![Undefined; 4];
        let input = [Off, Off];
        decoder(&input, &mut output);
        assert_eq!(output, vec![On, Off, Off, Off]);

        let input = [Off, On];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, On, Off, Off]);

        let input = [On, Off];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, Off, On, Off]);

        let input = [On, On];
        decoder(&input, &mut output);
        assert_eq!(output, vec![Off, Off, Off, On]);
    }
}
