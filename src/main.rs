#[macro_use] mod log;
mod cpu;
mod gpu;
mod timer;
mod chip;
mod keypad;

use chip::Chip;

fn main() {
    Chip::execute().unwrap();
}