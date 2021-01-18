use crate::{AudioSource, Decodable};
use bevy::asset::Asset;
use bevy::prelude::*;
use parking_lot::RwLock;
use std::{collections::VecDeque, fmt};

/// The external struct used to play audio
pub struct Audio<P = AudioSource>
where
    P: Asset + Decodable,
{
    pub queue: RwLock<VecDeque<Handle<P>>>,
    pub keys_queue: RwLock<VecDeque<Option<String>>>,
    pub pause_queue: RwLock<VecDeque<String>>,
}

impl<P: Asset> fmt::Debug for Audio<P>
where
    P: Decodable,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Audio").field("queue", &self.queue).finish()
    }
}

impl<P> Default for Audio<P>
where
    P: Asset + Decodable,
{
    fn default() -> Self {
        Self {
            queue: Default::default(),
            keys_queue: Default::default(),
            pause_queue: Default::default(),
        }
    }
}

impl<P> Audio<P>
where
    P: Asset + Decodable,
    <P as Decodable>::Decoder: rodio::Source + Send + Sync,
    <<P as Decodable>::Decoder as Iterator>::Item: rodio::Sample + Send + Sync,
{
    pub fn play_controlled(&self, audio_source: Handle<P>, key: String) {
        self.queue.write().push_front(audio_source);
        self.keys_queue.write().push_front(Some(key));
    }

    pub fn play(&self, audio_source: Handle<P>) {
        self.queue.write().push_front(audio_source);
        self.keys_queue.write().push_front(None);
    }

    pub fn pause(&self, key: String) {
        self.pause_queue.write().push_front(key);
    }
}
