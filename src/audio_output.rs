use crate::audio::Audio;
use bevy::prelude::*;

use crate::source::AudioSource;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sequence::Sequence;
use kira::sound::handle::SoundHandle;
use std::collections::HashMap;

pub struct AudioOutput {
    manager: AudioManager,
    sounds: HashMap<Handle<AudioSource>, SoundHandle>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        Self {
            manager: AudioManager::new(AudioManagerSettings::default()).unwrap(),
            sounds: HashMap::default(),
        }
    }
}

impl AudioOutput {
    pub fn play_queued(&mut self, audio_sources: &Assets<AudioSource>, audio: &mut Audio) {
        let mut queue = audio.queue.write();
        let len = queue.len();
        let mut i = 0;
        while i < len {
            let audio_source_handle = queue.pop_back().unwrap();

            if let Some(audio_source) = audio_sources.get(&audio_source_handle) {
                let handle = if let Some(handle) = self.sounds.get(&audio_source_handle) {
                    handle
                } else {
                    let sound = audio_source.sound.clone();
                    let handle = self.manager.add_sound(sound).unwrap();
                    self.sounds
                        .insert(audio_source_handle.clone(), handle.clone());
                    self.sounds.get(&audio_source_handle).unwrap()
                };
                let mut sequence = Sequence::<()>::new(Default::default());
                sequence.play(handle, Default::default());
                self.manager
                    .start_sequence(sequence, Default::default())
                    .unwrap();
            } else {
                // audio source hasn't loaded yet. add it back to the queue
                queue.push_front(audio_source_handle);
            }
            i += 1;
        }
    }
}

pub fn play_queued_audio_system(_world: &mut World, resources: &mut Resources) {
    let mut audio_output = resources.get_thread_local_mut::<AudioOutput>().unwrap();
    let mut audio = resources.get_mut::<Audio>().unwrap();
    if let Some(audio_sources) = resources.get::<Assets<AudioSource>>() {
        audio_output.play_queued(&*audio_sources, &mut *audio);
    }
}
