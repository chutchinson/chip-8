use std::time::{Instant, Duration};

pub struct Timer {
    frequency: Duration,
    clock: Instant,
    state: bool
}

impl Timer {

    pub fn new(frequency_ns: u32) -> Self {
        Timer {
            frequency: Duration::new(0, frequency_ns),
            clock: Instant::now(),
            state: false
        }
    }

    pub fn reset(&mut self) {
        self.state = false;
        self.clock = Instant::now();
    }

    pub fn active(&self) -> bool {
        self.state
    }

    pub fn tick(&mut self) {
        self.state = if self.clock.elapsed() >= self.frequency {
            self.clock = Instant::now();
            true
        }
        else {
            false
        }
    }

}