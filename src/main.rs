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

    // let vulkan = Instance::new(None, &InstanceExtensions::none(), None)
    //     .expect("failed to create graphics instance");

    // let device = PhysicalDevice::enumerate(&vulkan).next().expect("no device available");

    // println!("chip-8 gpu device: {}", device.name());

    // let mut events = EventsLoop::new();
    // let window = WindowBuilder::new()
    //     .with_title("chip-8")
    //     .build(&events);

    // let mut done = false;

    // while !done {
    //     events.poll_events(|event| {
    //         match event {
    //             Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => { 
    //                 done = true;
    //             },
    //             _ => {}
    //         }
    //     });
    // }

    // let program = fs::read("F://rom.ch8").unwrap();
    // let mut cpu = Cpu::new();

    // cpu.reset();
    // cpu.load(program.as_slice());

    // let n = 50;
    // for _ in 0..n {
    //     cpu.step();
    // }

}
