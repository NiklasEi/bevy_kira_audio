use crate::audio::{Audio, AudioCommands};
use bevy::prelude::*;

use crate::source::AudioSource;
use kira::arrangement::Arrangement;
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
    fn get_or_create_sound(
        &mut self,
        audio_source: &AudioSource,
        audio_source_handle: Handle<AudioSource>,
    ) -> SoundHandle {
        if let Some(handle) = self.sounds.get(&audio_source_handle) {
            return handle.clone();
        }

        let sound = audio_source.sound.clone();
        let handle = self.manager.add_sound(sound).unwrap();
        self.sounds.insert(audio_source_handle, handle.clone());
        handle
    }

    fn play(&mut self, sound_handle: &SoundHandle) {
        let mut sequence = Sequence::<()>::new(Default::default());
        sequence.play(sound_handle, Default::default());
        self.manager
            .start_sequence(sequence, Default::default())
            .unwrap();
    }

    fn play_looped(&mut self, sound_handle: &SoundHandle) {
        let arrangement = Arrangement::new_loop(sound_handle, Default::default());
        let mut arrangement_handle = self.manager.add_arrangement(arrangement).unwrap();
        if let Err(error) = arrangement_handle.play(Default::default()) {
            println!("Failed to play arrangement: {:?}", error);
        }
    }

    fn stop(&mut self) {
        for sound in self.sounds.values().into_iter() {
            if let Err(error) = self.manager.remove_sound(sound.id()) {
                println!("Failed to remove sound: {:?}", error);
            }
        }
    }

    pub fn run_queued_audio_commands(
        &mut self,
        audio_sources: &Assets<AudioSource>,
        audio: &mut Audio,
    ) {
        let mut commands = audio.commands.write();
        let len = commands.len();
        let mut i = 0;
        while i < len {
            let audio_command = commands.pop_back().unwrap();
            match &audio_command {
                AudioCommands::Play(play_settings) => {
                    if let Some(audio_source) = audio_sources.get(&play_settings.source) {
                        let sound_handle =
                            self.get_or_create_sound(audio_source, play_settings.source.clone());
                        if play_settings.looped {
                            self.play_looped(&sound_handle);
                        } else {
                            self.play(&sound_handle);
                        }
                    } else {
                        // audio source hasn't loaded yet. Add it back to the queue
                        commands.push_front(audio_command);
                    }
                }
                AudioCommands::Stop => {
                    self.stop();
                }
            }
            i += 1;
        }
    }
}

pub fn play_queued_audio_system(_world: &mut World, resources: &mut Resources) {
    let mut audio_output = resources.get_thread_local_mut::<AudioOutput>().unwrap();
    let mut audio = resources.get_mut::<Audio>().unwrap();
    if let Some(audio_sources) = resources.get::<Assets<AudioSource>>() {
        audio_output.run_queued_audio_commands(&*audio_sources, &mut *audio);
    }
}
