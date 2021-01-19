use crate::channel::Channel;
use crate::{Audio, AudioSource, Decodable};
use bevy::asset::Asset;
use bevy::ecs::bevy_utils::HashMap;
use bevy::prelude::*;
use std::marker::PhantomData;

/// Used internally to play audio on the current "audio device"
pub struct AudioOutput<P = AudioSource>
where
    P: Decodable,
{
    phantom: PhantomData<P>,
    channels: HashMap<Option<String>, Channel>,
}

impl<P> Default for AudioOutput<P>
where
    P: Decodable,
{
    fn default() -> Self {
        Self {
            phantom: PhantomData,
            channels: HashMap::default(),
        }
    }
}

impl<P> AudioOutput<P>
where
    P: Asset + Decodable,
    <P as Decodable>::Decoder: rodio::Source + Send + Sync,
    <<P as Decodable>::Decoder as Iterator>::Item: rodio::Sample + Send + Sync,
{
    fn play(&mut self, audio_source: &P, channel_key: Option<String>) {
        let channel = self.channels.get_mut(&channel_key);
        if let Some(channel) = channel {
            channel.add_audio(audio_source);
        } else {
            let mut channel = Channel::default();
            channel.add_audio(audio_source);
            self.channels.insert(channel_key, channel);
        }
    }

    fn pause(&mut self, channel_key: Option<String>) {
        let channel = self.channels.get_mut(&channel_key);
        if let Some(channel) = channel {
            channel.pause();
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
                self.play(audio_source, audio_source_key);
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
            self.pause(audio_source_key);

            i += 1;
        }

        let mut drop_queue = audio.drop_queue.write();
        let len = drop_queue.len();
        i = 0;
        while i < len {
            let audio_source_key = drop_queue.pop_back().unwrap();
            self.channels.remove(&audio_source_key);

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
