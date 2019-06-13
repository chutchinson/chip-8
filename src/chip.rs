use std::sync::Arc;
use std::rc::Rc;
use winit::{Window, WindowBuilder, Event, WindowEvent, EventsLoop};
use winit::dpi::{LogicalSize};
use wgpu::{SwapChain};
use crate::cpu::{Bus, Cpu};
use crate::gpu::Gpu;
use crate::timer::Timer;

const DEFAULT_CLOCK_RATE: u32 = 166666667;
const DEFAULT_WIDTH: u32 = 128;
const DEFAULT_HEIGHT: u32 = 64;
const SCALE: u32 = 4;

pub struct Chip {
    events: EventsLoop,
    window: Window,
    swapchain: SwapChain,
    bus: Bus,
    cpu: Cpu,
    gpu: Gpu
}

impl Chip {

    pub fn new() -> Self {
        let width = DEFAULT_WIDTH * SCALE;
        let height = DEFAULT_HEIGHT * SCALE;
        let events = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_title("chip-8")
            .with_dimensions(LogicalSize::new(width as f64, height as f64))
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
            width: width,
            height: height
        };
        let mut swapchain = device.create_swap_chain(&surface, &descriptor);
        Chip {
            events: events,
            window: window,
            swapchain: swapchain,
            bus: Bus {
                sound_timer: Timer::new(DEFAULT_CLOCK_RATE),
                delay_timer: Timer::new(DEFAULT_CLOCK_RATE)
            },
            cpu: Cpu::new(),
            gpu: Gpu::new(device)
        }
    }

    pub fn load(&mut self, rom: &[u8]) {
        self.reset();
        self.cpu.load(rom);
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
        self.bus.sound_timer.tick();
        self.bus.delay_timer.tick();
        self.cpu.cycle(&mut self.bus);
        let frame = self.swapchain.get_next_texture();
        self.gpu.render(&frame);
    }

}