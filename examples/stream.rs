use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioOutput, AudioPlugin, AudioStream, Frame};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(start_stream.system())
        .run();
}

// the note A
const A: f64 = 440.0;
const FREQ: f64 = 44_000.0;

#[derive(Debug)]
struct SineStream {
    t: f64,
}

impl AudioStream for SineStream {
    fn next(&mut self, _: f64) -> Frame {
        let increment = 2.0 * std::f64::consts::PI * A / FREQ;
        self.t += increment;

        let sample: f64 = self.t.sin();
        Frame::from_mono(sample as f32)
    }
}

fn start_stream(audio: Res<Audio>) {
    audio.stream(Box::new(SineStream { t: 0.0 }));
}
