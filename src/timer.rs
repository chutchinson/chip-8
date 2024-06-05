use std::time::{Instant, Duration};

pub struct Timer {
    frequency: Duration,
    clock: Instant,
    state: bool
}

impl Timer {

    pub fn new(frequency_ns: u32) -> Self {
        Timer {
            frequency: Duration::from_secs_f32(1.0 / 60.0), // 60 Hz
            clock: Instant::now(),
            state: false
        }
    }

    pub fn reset(&mut self) {
        self.clock = Instant::now();
    }

    pub fn tick(&mut self) -> bool {
        if self.clock.elapsed() >= self.frequency {
            self.reset();
            return true;
        }
        return false;
    }

}