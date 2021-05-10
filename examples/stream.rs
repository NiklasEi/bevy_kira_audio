use bevy::prelude::*;
use bevy_kira_audio::{AudioPlugin, AudioStream, AudioStreamPlugin, Frame, StreamedAudio};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(AudioStreamPlugin::<SineStream>::default())
        .add_startup_system(start_stream.system())
        .run();
}

// the note A
const A: f64 = 440.0;
const FREQ: f64 = 44_000.0;

#[derive(Debug, Default)]
struct SineStream {
    t: f64,
}

impl AudioStream for SineStream {
    fn next(&mut self, _: f64) -> Frame {
        let increment = 2.0 * std::f64::consts::PI * A / FREQ;
        self.t += increment;

        let sample: f64 = self.t.sin();
        Frame {
            left: sample as f32,
            right: sample as f32,
        }
    }
}

fn start_stream(audio: Res<StreamedAudio<SineStream>>) {
    audio.stream(SineStream { t: 0.0 });
}
