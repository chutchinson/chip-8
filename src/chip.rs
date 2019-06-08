use std::sync::Arc;
use winit::{Window, WindowBuilder, Event, WindowEvent, EventsLoop};
use wgpu::{SwapChain};
use crate::cpu::Cpu;
use crate::gpu::Gpu;

pub struct Chip {
    events: EventsLoop,
    window: Window,
    swapchain: SwapChain,
    cpu: Cpu,
    gpu: Gpu
}

impl Chip {

    pub fn new() -> Self {
        let events = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_title("chip-8")
            .build(&events)
            .unwrap();
        let instance = wgpu::Instance::new();
        let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
            power_preference: wgpu::PowerPreference::LowPower
        });
        let mut device = adapter.create_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false
            }
        });
        let surface = instance.create_surface(&window);
        let descriptor = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: 800 as u32,
            height: 600 as u32
        };
        let mut swapchain = device.create_swap_chain(&surface, &descriptor);
        Chip {
            events: events,
            window: window,
            swapchain: swapchain,
            cpu: Cpu::new(),
            gpu: Gpu::new(device)
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.gpu.reset();
    }

    pub fn halt(&mut self) {
        self.cpu.halt();
    }

    pub fn run(&mut self) {
        let mut running = true;
        while running {
            self.events.poll_events(|event| {
                match event {
                    Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                        running = false;
                    },
                    _ => {}
                }
            });
            self.cycle();
        }
        self.halt();
    }

    fn cycle(&mut self) {
        self.cpu.cycle();
        let frame = self.swapchain.get_next_texture();
        self.gpu.render(&frame);
    }

}