use crate::audio_source::AudioSource;
use bevy::prelude::Handle;
use parking_lot::RwLock;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Audio {
    pub queue: RwLock<VecDeque<Handle<AudioSource>>>,
}

impl Audio {
    pub fn play(&self, audio_source: Handle<AudioSource>) {
        self.queue.write().push_front(audio_source);
    }
}
