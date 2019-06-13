#[macro_use] mod log;
mod cpu;
mod gpu;
mod timer;
mod chip;

use chip::Chip;

fn main() {
    let rom = std::fs::read("F://rom.ch8").unwrap();
    let mut chip = Chip::new();

    chip.reset();
    chip.load(&rom[0..]);
    chip.run();
}
