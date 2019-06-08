#[macro_use] mod log;
mod cpu;
mod gpu;
mod chip;

use chip::Chip;

use std::fs;
use cpu::Cpu;
use winit::{EventsLoop, Window, WindowBuilder, Event, WindowEvent, ControlFlow};
use winit::dpi::{LogicalSize};

fn main() {
    let mut chip = Chip::new();
    
    chip.reset();
    chip.run();
}
