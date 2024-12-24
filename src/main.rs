mod and;
mod bit;
mod decoder;
mod nand;
mod not;
mod register;

use digital_component::BitState;

fn main() {
    let _ = bit::bit;
    let _ = not::not;
    let _ = and::and;
    let _ = register::register;
    let _ = decoder::decoder_2_to_4;
}
