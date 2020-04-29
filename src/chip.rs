use crate::cpu::{Cpu, CpuContext};
use crate::gpu::Gpu;
use crate::timer::Timer;

use coffee::{Game, Result};
use coffee::load::{Task};
use coffee::graphics::{Frame, Window, WindowSettings};

const DEFAULT_CLOCK_RATE: u32 = 166666667;
const DEFAULT_WIDTH: u32 = 128;
const DEFAULT_HEIGHT: u32 = 64;
const SCALE: u32 = 4;

pub struct Chip {
    sound_timer: Timer,
    delay_timer: Timer,
    gpu: Gpu,
    cpu: Cpu,
}

impl Game for Chip {
    type Input = ();
    type LoadingScreen = ();

    fn load(_window: &Window) -> Task<Chip> {
        let rom = std::fs::read("E://maze.ch8").unwrap();
        let mut chip = Chip::new();
        chip.load(&rom[0..]);
        Task::succeed(|| chip)
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &coffee::Timer) {
        self.cycle(frame);
    }
}

impl Chip {

    pub fn execute() -> Result<()> {
        let width = DEFAULT_WIDTH * SCALE;
        let height = DEFAULT_HEIGHT * SCALE;
        Chip::run(WindowSettings {
            title: String::from("chip-8"),
            size: (width, height),
            resizable: false,
            fullscreen: false,
            maximized: false
        })
    }

    pub fn new() -> Self {
        Chip {
            sound_timer: Timer::new(DEFAULT_CLOCK_RATE),
            delay_timer: Timer::new(DEFAULT_CLOCK_RATE),
            cpu: Cpu::new(),
            gpu: Gpu::new()
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
    
    pub fn cycle(&mut self, frame: &mut Frame) {
        self.sound_timer.tick();
        self.delay_timer.tick();

        let mut ctx = CpuContext {
            sound_timer: &mut self.sound_timer,
            delay_timer: &mut self.delay_timer,
            gpu: &mut self.gpu
        };

        self.cpu.cycle(&mut ctx);
        self.gpu.render(frame);
    }

}