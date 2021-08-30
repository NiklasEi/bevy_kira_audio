use bevy::prelude::*;
use bevy_kira_audio::{AudioPlugin, AudioStream, AudioStreamPlugin, Frame, StreamedAudio};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(AudioStreamPlugin::<SineStream>::default())
        .add_startup_system(start_stream.system())
        .run();
}

#[derive(Debug, Default)]
struct SineStream {
    t: f64,
    note: f64,
    frequency: f64,
}

impl AudioStream for SineStream {
    fn next(&mut self, _: f64) -> Frame {
        self.t += 2.0 * std::f64::consts::PI * self.note / self.frequency;
        Frame::from_mono(self.t.sin() as f32)
    }
}

fn start_stream(audio: Res<StreamedAudio<SineStream>>) {
    audio.stream(SineStream {
        t: 0.0,
        note: 440.0,
        frequency: 44_000.0,
    });
}
