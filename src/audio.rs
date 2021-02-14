use crate::source::AudioSource;
use bevy::prelude::Handle;
use parking_lot::RwLock;
use std::collections::VecDeque;

pub struct PlayAudioSettings {
    pub source: Handle<AudioSource>,
    pub looped: bool,
}

#[derive(Default)]
pub struct Audio {
    pub queue: RwLock<VecDeque<PlayAudioSettings>>,
}

impl Audio {
    pub fn play(&self, audio_source: Handle<AudioSource>) {
        self.queue.write().push_front(PlayAudioSettings {
            source: audio_source,
            looped: false,
        });
    }

    pub fn play_looped(&self, audio_source: Handle<AudioSource>) {
        self.queue.write().push_front(PlayAudioSettings {
            source: audio_source,
            looped: true,
        });
    }
}
