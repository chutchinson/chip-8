use crate::cpu::{Cpu, CpuContext};
use crate::gpu::Gpu;
use crate::timer::Timer;
use crate::keypad::Keypad;

use std::collections::HashSet;

use coffee::{Game, Result};
use coffee::load::{Task};
use coffee::input::{Input};
use coffee::input::keyboard::{KeyCode};
use coffee::graphics::{Frame, Window, WindowSettings};

const DEFAULT_CLOCK_RATE: u32 = 166666667;
const DEFAULT_WIDTH: u32 = 64;
const DEFAULT_HEIGHT: u32 = 32;
const SCALE: u32 = 10;

pub struct Chip {
    sound_timer: Timer,
    delay_timer: Timer,
    gpu: Gpu,
    cpu: Cpu,
    keypad: Keypad,
    autorun: bool,
    step: bool
}

use coffee::input::KeyboardAndMouse;

impl Game for Chip {
    type Input = KeyboardAndMouse;
    type LoadingScreen = ();

    fn load(_window: &Window) -> Task<Chip> {
        let rom = std::fs::read("E://trip.ch8").unwrap();
        let mut chip = Chip::new();
        chip.load(&rom[0..]);
        Task::succeed(|| chip)
    }

    fn interact(&mut self, input: &mut Self::Input, _window: &mut Window) {
        let mapping = vec![
            KeyCode::Q, KeyCode::W, KeyCode::E,
            KeyCode::A, KeyCode::A, KeyCode::D, 
            KeyCode::Z, KeyCode::X, KeyCode::C,
        ];
        let keyboard = input.keyboard();
        for x in 0..mapping.len() {
            let key = x as usize;
            let pressed = keyboard.is_key_pressed(mapping[key]);
            self.keypad.set(key, pressed);
        }
        if keyboard.was_key_released(KeyCode::F6) {
            self.step = true;
        }
        if keyboard.was_key_released(KeyCode::F1) {
            self.autorun = !self.autorun;
        }
        if keyboard.was_key_released(KeyCode::F2) {
            self.gpu.reset();
            self.cpu.reset();
        }
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &coffee::Timer) {
        if self.step || self.autorun {
//            self.dump();
            self.cycle(frame);
            self.step = false;
        }
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
            gpu: Gpu::new(),
            keypad: Keypad::new(),
            step: false,
            autorun: true
        }
    }
    
    pub fn dump(&self) {
        self.cpu.dump();
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
            opcode: 0,
            sound_timer: &mut self.sound_timer,
            delay_timer: &mut self.delay_timer,
            gpu: &mut self.gpu
        };

        self.cpu.cycle(&mut ctx);
        self.gpu.render(frame);
    }

}