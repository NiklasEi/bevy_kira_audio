use crate::Decodable;
use bevy::asset::Asset;
use rodio::{OutputStream, OutputStreamHandle, Sink};

pub struct Channel {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: Vec<Sink>,
    paused: bool,
}

impl Default for Channel {
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Channel {
            _stream: stream,
            stream_handle,
            sinks: vec![],
            paused: false,
        }
    }
}

impl Channel {
    pub fn pause(&mut self) {
        for sink in self.sinks.iter() {
            sink.pause();
        }
        self.paused = true;
    }

    pub fn play(&mut self) {
        for sink in self.sinks.iter() {
            sink.play();
        }
        self.paused = false;
    }

    pub fn add_audio<P>(&mut self, audio_source: &P)
    where
        P: Asset + Decodable,
        <P as Decodable>::Decoder: rodio::Source + Send + Sync,
        <<P as Decodable>::Decoder as Iterator>::Item: rodio::Sample + Send + Sync,
    {
        for sink in self.sinks.iter_mut() {
            if sink.empty() {
                sink.append(audio_source.decoder());
                return;
            }
        }
        let sink = Sink::try_new(&self.stream_handle).unwrap();
        sink.append(audio_source.decoder());
        if self.paused {
            sink.pause();
        }
        self.sinks.push(sink);
    }
}
