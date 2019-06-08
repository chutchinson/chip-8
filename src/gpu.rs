use std::rc::Rc;
use std::sync::Arc;
use winit::{Window, WindowBuilder, EventsLoop};
use wgpu::{SwapChain, Device, SwapChainOutput};

pub struct Gpu {
    device: Device,
    vram: [u8; 4096]
}

impl Gpu {

    pub fn new(device: Device) -> Self {
        Gpu {
            device: device,
            vram: [0; 4096]
        }
    }

    pub fn reset(&mut self) {
        log!("[gpu] reset");
    }

    pub fn render(&mut self, frame: &SwapChainOutput) {
        
        println!("[gpu] render");
    }

}