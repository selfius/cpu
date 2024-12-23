mod and;
mod bit;
mod nand;
mod not;
mod register;

use digital_component::BitState;

fn main() {
    let _ = bit::bit;
    let _ = not::not;
    let _ = and::and;
    let _ = register::register;
}
