use crate::audio::{Audio, AudioCommands, PlayAudioSettings};
use bevy::prelude::*;

use crate::channel::AudioChannel;
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
    instances: HashMap<AudioChannel, Vec<InstanceHandle>>,
    channels: HashMap<AudioChannel, ChannelState>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        Self {
            manager: AudioManager::new(AudioManagerSettings::default()).unwrap(),
            sounds: HashMap::default(),
            arrangements: HashMap::default(),
            instances: HashMap::default(),
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

    fn play_arrangement(
        &mut self,
        mut arrangement_handle: ArrangementHandle,
        channel: &AudioChannel,
    ) {
        let play_result = arrangement_handle.play(Default::default());
        if let Err(error) = play_result {
            println!("Failed to play arrangement: {:?}", error);
            return;
        }
        let mut instance_handle = play_result.unwrap();
        if let Some(channel_state) = self.channels.get(&channel) {
            if let Err(error) = instance_handle.set_volume(channel_state.volume) {
                println!("Failed to set volume for instance: {:?}", error);
            }
            if let Err(error) = instance_handle.set_pitch(channel_state.pitch) {
                println!("Failed to set volume for instance: {:?}", error);
            }
            if let Err(error) = instance_handle.set_panning(channel_state.panning) {
                println!("Failed to set volume for instance: {:?}", error);
            }
        }
        if let Some(instance_handles) = self.instances.get_mut(&channel) {
            instance_handles.push(instance_handle);
        } else {
            self.instances
                .insert(channel.clone(), vec![instance_handle]);
        }
    }

    fn play(&mut self, sound_handle: &SoundHandle, channel: &AudioChannel) -> ArrangementHandle {
        let mut arrangement = Arrangement::new(ArrangementSettings::new().cooldown(0.0));
        arrangement.add_clip(SoundClip::new(sound_handle, 0.0));
        let arrangement_handle = self.manager.add_arrangement(arrangement).unwrap();

        self.play_arrangement(arrangement_handle.clone(), channel);
        arrangement_handle
    }

    fn play_looped(
        &mut self,
        sound_handle: &SoundHandle,
        channel: &AudioChannel,
    ) -> ArrangementHandle {
        let arrangement = Arrangement::new_loop(sound_handle, Default::default());
        let arrangement_handle = self.manager.add_arrangement(arrangement).unwrap();

        self.play_arrangement(arrangement_handle.clone(), channel);
        arrangement_handle
    }

    fn stop(&mut self, channel_id: AudioChannel) {
        if let Some(instances) = self.instances.get_mut(&channel_id) {
            for mut instance in instances.drain(..) {
                if let Err(error) = instance.stop(StopInstanceSettings::default()) {
                    println!("Failed to stop instance: {:?}", error);
                }
            }
        }
    }

    fn pause(&mut self, channel_id: AudioChannel) {
        if let Some(instances) = self.instances.get_mut(&channel_id) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.pause(PauseInstanceSettings::default()) {
                    println!("Failed to pause instance: {:?}", error);
                }
            }
        }
    }

    fn resume(&mut self, channel_id: AudioChannel) {
        if let Some(instances) = self.instances.get_mut(&channel_id) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.resume(ResumeInstanceSettings::default()) {
                    println!("Failed to resume instance: {:?}", error);
                }
            }
        }
    }

    fn set_volume(&mut self, channel_id: AudioChannel, volume: f64) {
        if let Some(instances) = self.instances.get_mut(&channel_id) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.set_volume(volume) {
                    println!("Failed to set volume for instance: {:?}", error);
                }
            }
        }
        if let Some(mut channel_state) = self.channels.get_mut(&channel_id) {
            channel_state.volume = volume;
        } else {
            let mut channel_state = ChannelState::default();
            channel_state.volume = volume;
            self.channels.insert(channel_id, channel_state);
        }
    }

    fn set_panning(&mut self, channel_id: AudioChannel, panning: f64) {
        if let Some(instances) = self.instances.get_mut(&channel_id) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.set_panning(panning) {
                    println!("Failed to set panning for instance: {:?}", error);
                }
            }
        }
        if let Some(mut channel_state) = self.channels.get_mut(&channel_id) {
            channel_state.panning = panning;
        } else {
            let mut channel_state = ChannelState::default();
            channel_state.panning = panning;
            self.channels.insert(channel_id, channel_state);
        }
    }

    fn set_pitch(&mut self, channel_id: AudioChannel, pitch: f64) {
        if let Some(instances) = self.instances.get_mut(&channel_id) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.set_pitch(pitch) {
                    println!("Failed to set pitch for instance: {:?}", error);
                }
            }
        }
        if let Some(mut channel_state) = self.channels.get_mut(&channel_id) {
            channel_state.pitch = pitch;
        } else {
            let mut channel_state = ChannelState::default();
            channel_state.pitch = pitch;
            self.channels.insert(channel_id, channel_state);
        }
    }

    pub(crate) fn run_queued_audio_commands(
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
                        if self.arrangements.contains_key(play_settings) {
                            self.play_arrangement(
                                self.arrangements.get(play_settings).unwrap().clone(),
                                &channel_id,
                            );
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
                    self.stop(channel_id);
                }
                AudioCommands::Pause => {
                    self.pause(channel_id);
                }
                AudioCommands::Resume => {
                    self.resume(channel_id);
                }
                AudioCommands::SetVolume(volume) => {
                    self.set_volume(channel_id, *volume as f64);
                }
                AudioCommands::SetPanning(panning) => {
                    self.set_panning(channel_id, *panning as f64);
                }
                AudioCommands::SetPitch(pitch) => {
                    self.set_pitch(channel_id, *pitch as f64);
                }
            }
            i += 1;
        }
    }
}

struct ChannelState {
    volume: f64,
    pitch: f64,
    panning: f64,
}

impl Default for ChannelState {
    fn default() -> Self {
        ChannelState {
            volume: 1.0,
            pitch: 1.0,
            panning: 0.5,
        }
    }
}

pub fn play_queued_audio_system(_world: &mut World, resources: &mut Resources) {
    let mut audio_output = resources.get_non_send_mut::<AudioOutput>().unwrap();
    let mut audio = resources.get_mut::<Audio>().unwrap();
    if let Some(audio_sources) = resources.get::<Assets<AudioSource>>() {
        audio_output.run_queued_audio_commands(&*audio_sources, &mut *audio);
    }
}
