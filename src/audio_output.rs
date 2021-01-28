use crate::audio::Audio;
use bevy::prelude::*;
use kira::arrangement::Arrangement;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sequence::Sequence;
use kira::sound::handle::SoundHandle;
use kira::sound::SoundSettings;
use std::collections::HashMap;

pub struct AudioOutput {
    manager: AudioManager,
    sound_cache: HashMap<String, SoundHandle>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        Self {
            manager: AudioManager::new(AudioManagerSettings::default()).unwrap(),
            sound_cache: HashMap::default(),
        }
    }
}

impl AudioOutput {
    pub fn play_queued(&mut self, audio: &mut Audio) {
        let mut queue = audio.queue.write();
        let len = queue.len();
        let mut i = 0;
        while i < len {
            let path = queue.pop_back().unwrap();
            let mut sequence = Sequence::<()>::new(Default::default());
            if let Some(sound) = self.sound_cache.get(&path) {
                sequence.play(sound, Default::default());
            } else {
                let sound = self
                    .manager
                    .load_sound(path.clone(), SoundSettings::default())
                    .unwrap();
                sequence.play(&sound, Default::default());
                self.sound_cache.insert(path, sound);
            }
            self.manager
                .start_sequence(sequence, Default::default())
                .unwrap();

            i += 1;
        }

        let mut loop_queue = audio.loop_queue.write();
        let len = loop_queue.len();
        let mut i = 0;
        while i < len {
            let path = loop_queue.pop_back().unwrap();
            let mut sequence = Sequence::<()>::new(Default::default());
            if let Some(sound) = self.sound_cache.get(&path) {
                let loop_id = self
                    .manager
                    .add_arrangement(Arrangement::new_loop(sound, Default::default()))
                    .unwrap();
                sequence.play(&loop_id, Default::default());
            } else {
                let sound = self
                    .manager
                    .load_sound(path.clone(), SoundSettings::default())
                    .unwrap();
                let loop_id = self
                    .manager
                    .add_arrangement(Arrangement::new_loop(&sound, Default::default()))
                    .unwrap();
                sequence.play(&loop_id, Default::default());
                self.sound_cache.insert(path, sound);
            }
            self.manager
                .start_sequence(sequence, Default::default())
                .unwrap();
            i += 1;
        }
    }
}

pub fn play_queued_audio_system(_world: &mut World, resources: &mut Resources) {
    let mut audio_output = resources.get_thread_local_mut::<AudioOutput>().unwrap();
    let mut audio = resources.get_mut::<Audio>().unwrap();

    audio_output.play_queued(&mut *audio);
}
