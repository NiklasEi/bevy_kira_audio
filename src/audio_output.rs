use crate::{Audio, AudioSource, Decodable};
use bevy::asset::Asset;
use bevy::ecs::bevy_utils::HashMap;
use bevy::prelude::*;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::marker::PhantomData;

/// Used internally to play audio on the current "audio device"
pub struct AudioOutput<P = AudioSource>
where
    P: Decodable,
{
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    phantom: PhantomData<P>,
    sinks: HashMap<String, Sink>,
}

impl<P> Default for AudioOutput<P>
where
    P: Decodable,
{
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            _stream: stream,
            stream_handle,
            phantom: PhantomData,
            sinks: Default::default(),
        }
    }
}

impl<P> AudioOutput<P>
where
    P: Asset + Decodable,
    <P as Decodable>::Decoder: rodio::Source + Send + Sync,
    <<P as Decodable>::Decoder as Iterator>::Item: rodio::Sample + Send + Sync,
{
    fn play_source(&mut self, audio_source: &P, key: Option<String>) {
        let sink = Sink::try_new(&self.stream_handle).unwrap();
        sink.append(audio_source.decoder());
        if let Some(key) = key {
            self.sinks.insert(key, sink);
        } else {
            sink.detach();
        }
    }

    fn pause_source(&mut self, key: String) {
        let sink = self.sinks.get_mut(&key);
        if let Some(sink) = sink {
            sink.pause();
        }
    }

    fn try_play_queued(&mut self, audio_sources: &Assets<P>, audio: &mut Audio<P>) {
        let mut queue = audio.queue.write();
        let mut key_queue = audio.keys_queue.write();
        let len = queue.len();
        let mut i = 0;
        while i < len {
            let audio_source_handle = queue.pop_back().unwrap();
            if let Some(audio_source) = audio_sources.get(&audio_source_handle) {
                let audio_source_key = key_queue.pop_back().unwrap();
                self.play_source(audio_source, audio_source_key);
            } else {
                // audio source hasn't loaded yet. add it back to the queue
                queue.push_front(audio_source_handle);
            }
            i += 1;
        }

        let mut pause_queue = audio.pause_queue.write();
        let len = pause_queue.len();
        i = 0;
        while i < len {
            let audio_source_key = pause_queue.pop_back().unwrap();
            self.pause_source(audio_source_key);

            i += 1;
        }
    }
}

/// Plays audio currently queued in the [Audio] resource through the [AudioOutput] resource
pub fn play_queued_audio_system<P: Asset>(_world: &mut World, resources: &mut Resources)
where
    P: Decodable,
    <P as Decodable>::Decoder: rodio::Source + Send + Sync,
    <<P as Decodable>::Decoder as Iterator>::Item: rodio::Sample + Send + Sync,
{
    let mut audio_output = resources.get_thread_local_mut::<AudioOutput<P>>().unwrap();
    let mut audio = resources.get_mut::<Audio<P>>().unwrap();

    if let Some(audio_sources) = resources.get::<Assets<P>>() {
        audio_output.try_play_queued(&*audio_sources, &mut *audio);
    }
}
