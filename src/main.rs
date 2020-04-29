#[macro_use] mod log;
mod cpu;
mod gpu;
mod timer;
mod chip;

use chip::Chip;

fn main() {
    Chip::execute().unwrap();
}