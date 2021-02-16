use crate::audio::{Audio, AudioCommands, PlayAudioSettings};
use bevy::prelude::*;

use crate::channel::ChannelId;
use crate::source::AudioSource;
use kira::arrangement::handle::ArrangementHandle;
use kira::arrangement::{Arrangement, ArrangementSettings, SoundClip};
use kira::instance::handle::InstanceHandle;
use kira::instance::{PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings};
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::handle::SoundHandle;
use std::collections::HashMap;

pub struct AudioOutput {
    manager: AudioManager,
    sounds: HashMap<Handle<AudioSource>, SoundHandle>,
    arrangements: HashMap<PlayAudioSettings, ArrangementHandle>,
    channels: HashMap<ChannelId, Vec<InstanceHandle>>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        Self {
            manager: AudioManager::new(AudioManagerSettings::default()).unwrap(),
            sounds: HashMap::default(),
            arrangements: HashMap::default(),
            channels: HashMap::default(),
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

    fn play_arrangement(&mut self, mut arrangement_handle: ArrangementHandle, channel: &ChannelId) {
        let play_result = arrangement_handle.play(Default::default());
        if let Err(error) = play_result {
            println!("Failed to play arrangement: {:?}", error);
            return;
        }
        let instance_handle = play_result.unwrap();
        if let Some(instance_handles) = self.channels.get_mut(&channel) {
            instance_handles.push(instance_handle);
        } else {
            self.channels.insert(channel.clone(), vec![instance_handle]);
        }
    }

    fn play(&mut self, sound_handle: &SoundHandle, channel: &ChannelId) -> ArrangementHandle {
        let mut arrangement = Arrangement::new(ArrangementSettings::new());
        arrangement.add_clip(SoundClip::new(sound_handle, 0.0));
        let arrangement_handle = self.manager.add_arrangement(arrangement).unwrap();

        self.play_arrangement(arrangement_handle.clone(), channel);
        arrangement_handle
    }

    fn play_looped(
        &mut self,
        sound_handle: &SoundHandle,
        channel: &ChannelId,
    ) -> ArrangementHandle {
        let arrangement = Arrangement::new_loop(sound_handle, Default::default());
        let arrangement_handle = self.manager.add_arrangement(arrangement).unwrap();

        self.play_arrangement(arrangement_handle.clone(), channel);
        arrangement_handle
    }

    fn stop(&mut self) {
        for (_channel_id, mut instances) in self.channels.drain() {
            for mut instance in instances.drain(..) {
                if let Err(error) = instance.stop(StopInstanceSettings::default()) {
                    println!("Failed to stop instance: {:?}", error);
                }
            }
        }
    }

    fn pause(&mut self) {
        for (_channel_id, instances) in self.channels.iter_mut() {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.pause(PauseInstanceSettings::default()) {
                    println!("Failed to pause instance: {:?}", error);
                }
            }
        }
    }

    fn resume(&mut self) {
        for (_channel_id, instances) in self.channels.iter_mut() {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.resume(ResumeInstanceSettings::default()) {
                    println!("Failed to resume instance: {:?}", error);
                }
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
            let (audio_command, channel_id) = commands.pop_back().unwrap();
            match &audio_command {
                AudioCommands::Play(play_settings) => {
                    if let Some(audio_source) = audio_sources.get(&play_settings.source) {
                        let sound_handle =
                            self.get_or_create_sound(audio_source, play_settings.source.clone());
                        if let Some(arrangement_handle) = self.arrangements.get_mut(play_settings) {
                            if let Err(error) = arrangement_handle.play(Default::default()) {
                                println!("Failed to play arrangement: {:?}", error);
                            }
                        } else {
                            let arrangement_handle = if play_settings.looped {
                                self.play_looped(&sound_handle, &channel_id)
                            } else {
                                self.play(&sound_handle, &channel_id)
                            };
                            self.arrangements
                                .insert(play_settings.clone(), arrangement_handle);
                        }
                    } else {
                        // audio source hasn't loaded yet. Add it back to the queue
                        commands.push_front((audio_command, channel_id));
                    }
                }
                AudioCommands::Stop => {
                    self.stop();
                }
                AudioCommands::Pause => {
                    self.pause();
                }
                AudioCommands::Resume => {
                    self.resume();
                }
            }
            i += 1;
        }
        self.manager.free_unused_resources();
    }
}

pub fn play_queued_audio_system(_world: &mut World, resources: &mut Resources) {
    let mut audio_output = resources.get_thread_local_mut::<AudioOutput>().unwrap();
    let mut audio = resources.get_mut::<Audio>().unwrap();
    if let Some(audio_sources) = resources.get::<Assets<AudioSource>>() {
        audio_output.run_queued_audio_commands(&*audio_sources, &mut *audio);
    }
}
