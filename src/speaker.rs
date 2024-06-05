use std::time::Duration;

use rodio::{Device, OutputStream, Sink};
use rodio::source::{Source, SineWave};

pub struct Speaker {
    output: OutputStream,
    sink: Sink
}

impl Speaker {

    pub fn new(freq: f32) -> Self {
        let (output, handle) = OutputStream::try_default().expect("Could not find audio device");
        let sink = Sink::try_new(&handle).expect("Audio sink open failed");
        let source = SineWave::new(freq).repeat_infinite();
        sink.append(source);
        sink.set_volume(0.1);
        sink.pause();
        Speaker {
            output,
            sink
        }
    }

    pub fn render(&mut self, state: bool) {
        if state { self.sink.play() } else { self.sink.pause() };
    }
}