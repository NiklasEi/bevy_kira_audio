use crate::audio::{Audio, AudioCommands, PlayAudioSettings};
use bevy::prelude::*;

use crate::channel::AudioChannel;
use crate::source::AudioSource;
use crate::stream::{InternalAudioStream, StreamCommands, StreamedAudio};
use crate::AudioStream;
use kira::arrangement::handle::ArrangementHandle;
use kira::arrangement::{Arrangement, ArrangementSettings, SoundClip};
use kira::audio_stream::AudioStreamId;
use kira::instance::handle::InstanceHandle;
use kira::instance::{PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings};
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::mixer::TrackIndex;
use kira::sound::handle::SoundHandle;
use std::collections::HashMap;

pub struct AudioOutput {
    manager: AudioManager,
    sounds: HashMap<Handle<AudioSource>, SoundHandle>,
    arrangements: HashMap<PlayAudioSettings, ArrangementHandle>,
    streams: HashMap<AudioChannel, Vec<AudioStreamId>>,
    instances: HashMap<AudioChannel, Vec<InstanceHandle>>,
    channels: HashMap<AudioChannel, ChannelState>,
}

/// Output
impl Default for AudioOutput {
    fn default() -> Self {
        Self {
            manager: AudioManager::new(AudioManagerSettings::default())
                .expect("Failed to initialize AudioManager"),
            sounds: HashMap::default(),
            arrangements: HashMap::default(),
            streams: HashMap::default(),
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
        let handle = self
            .manager
            .add_sound(sound)
            .expect("Failed to add sound to the AudioManager");
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
            if let Err(error) = instance_handle.set_playback_rate(channel_state.playback_rate) {
                println!("Failed to set playback rate for instance: {:?}", error);
            }
            if let Err(error) = instance_handle.set_panning(channel_state.panning) {
                println!("Failed to set panning for instance: {:?}", error);
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
        let arrangement_handle = self
            .manager
            .add_arrangement(arrangement)
            .expect("Failed to add arrangement to the AudioManager");

        self.play_arrangement(arrangement_handle.clone(), channel);
        arrangement_handle
    }

    fn play_looped(
        &mut self,
        sound_handle: &SoundHandle,
        channel: &AudioChannel,
    ) -> ArrangementHandle {
        let arrangement = Arrangement::new_loop(sound_handle, Default::default());
        let arrangement_handle = self
            .manager
            .add_arrangement(arrangement)
            .expect("Failed to add arrangement to the AudioManager");

        self.play_arrangement(arrangement_handle.clone(), channel);
        arrangement_handle
    }

    fn stop(&mut self, channel: AudioChannel) {
        if let Some(instances) = self.instances.get_mut(&channel) {
            for mut instance in instances.drain(..) {
                if let Err(error) = instance.stop(StopInstanceSettings::default()) {
                    println!("Failed to stop instance: {:?}", error);
                }
            }
        }
    }

    fn pause(&mut self, channel: AudioChannel) {
        if let Some(instances) = self.instances.get_mut(&channel) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.pause(PauseInstanceSettings::default()) {
                    println!("Failed to pause instance: {:?}", error);
                }
            }
        }
    }

    fn resume(&mut self, channel: AudioChannel) {
        if let Some(instances) = self.instances.get_mut(&channel) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.resume(ResumeInstanceSettings::default()) {
                    println!("Failed to resume instance: {:?}", error);
                }
            }
        }
    }

    fn set_volume(&mut self, channel: AudioChannel, volume: f64) {
        if let Some(instances) = self.instances.get_mut(&channel) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.set_volume(volume) {
                    println!("Failed to set volume for instance: {:?}", error);
                }
            }
        }
        if let Some(mut channel_state) = self.channels.get_mut(&channel) {
            channel_state.volume = volume;
        } else {
            let channel_state = ChannelState {
                volume,
                ..Default::default()
            };
            self.channels.insert(channel, channel_state);
        }
    }

    fn set_panning(&mut self, channel: AudioChannel, panning: f64) {
        if let Some(instances) = self.instances.get_mut(&channel) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.set_panning(panning) {
                    println!("Failed to set panning for instance: {:?}", error);
                }
            }
        }
        if let Some(mut channel_state) = self.channels.get_mut(&channel) {
            channel_state.panning = panning;
        } else {
            let channel_state = ChannelState {
                panning,
                ..Default::default()
            };
            self.channels.insert(channel, channel_state);
        }
    }

    fn set_playback_rate(&mut self, channel: AudioChannel, playback_rate: f64) {
        if let Some(instances) = self.instances.get_mut(&channel) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.set_playback_rate(playback_rate) {
                    println!("Failed to set playback rate for instance: {:?}", error);
                }
            }
        }
        if let Some(mut channel_state) = self.channels.get_mut(&channel) {
            channel_state.playback_rate = playback_rate;
        } else {
            let channel_state = ChannelState {
                playback_rate,
                ..Default::default()
            };
            self.channels.insert(channel, channel_state);
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
            let (audio_command, channel) = commands.pop_back().unwrap();
            match &audio_command {
                AudioCommands::Play(play_settings) => {
                    if let Some(audio_source) = audio_sources.get(&play_settings.source) {
                        if self.arrangements.contains_key(play_settings) {
                            self.play_arrangement(
                                self.arrangements.get(play_settings).unwrap().clone(),
                                &channel,
                            );
                        } else {
                            let sound_handle = self
                                .get_or_create_sound(audio_source, play_settings.source.clone());
                            let arrangement_handle = if play_settings.looped {
                                self.play_looped(&sound_handle, &channel)
                            } else {
                                self.play(&sound_handle, &channel)
                            };
                            self.arrangements
                                .insert(play_settings.clone(), arrangement_handle);
                        }
                    } else {
                        // audio source hasn't loaded yet. Add it back to the queue
                        commands.push_front((audio_command, channel));
                    }
                }
                AudioCommands::Stop => {
                    self.stop(channel);
                }
                AudioCommands::Pause => {
                    self.pause(channel);
                }
                AudioCommands::Resume => {
                    self.resume(channel);
                }
                AudioCommands::SetVolume(volume) => {
                    self.set_volume(channel, *volume as f64);
                }
                AudioCommands::SetPanning(panning) => {
                    self.set_panning(channel, *panning as f64);
                }
                AudioCommands::SetPlaybackRate(playback_rate) => {
                    self.set_playback_rate(channel, *playback_rate as f64);
                }
            }
            i += 1;
        }
    }

    fn start_stream<T: kira::audio_stream::AudioStream>(
        &mut self,
        stream: T,
        channel: AudioChannel,
    ) {
        let stream_id = self
            .manager
            .add_stream(stream, TrackIndex::Main)
            .expect("Failed to play audio stream");
        if let Some(stream_ids) = self.streams.get_mut(&channel) {
            stream_ids.push(stream_id);
        } else {
            self.streams.insert(channel.clone(), vec![stream_id]);
        }
    }

    fn stop_streams(&mut self, channel: AudioChannel) {
        if let Some(stream_ids) = self.streams.get_mut(&channel) {
            for stream_id in stream_ids.drain(..) {
                if let Err(error) = self.manager.remove_stream(stream_id) {
                    println!("Failed to stop stream: {:?}", error);
                }
            }
        }
    }

    pub(crate) fn stream_audio<T: AudioStream>(&mut self, audio: &mut StreamedAudio<T>) {
        let mut commands = audio.commands.write();
        let len = commands.len();
        let mut i = 0;
        while i < len {
            let (stream, channel) = commands.pop_back().unwrap();
            match stream {
                StreamCommands::Play(stream) => {
                    let audio_stream = InternalAudioStream::new(stream);
                    self.start_stream(audio_stream, channel);
                }
                StreamCommands::Stop => self.stop_streams(channel),
            }
            i += 1;
        }
    }
}

struct ChannelState {
    volume: f64,
    playback_rate: f64,
    panning: f64,
}

impl Default for ChannelState {
    fn default() -> Self {
        ChannelState {
            volume: 1.0,
            playback_rate: 1.0,
            panning: 0.5,
        }
    }
}

pub fn play_queued_audio_system(world: &mut World) {
    let world = world.cell();

    let mut audio_output = world.get_non_send_mut::<AudioOutput>().unwrap();
    let mut audio = world.get_resource_mut::<Audio>().unwrap();
    if let Some(audio_sources) = world.get_resource::<Assets<AudioSource>>() {
        audio_output.run_queued_audio_commands(&*audio_sources, &mut *audio);
    };
}

pub fn stream_audio_system<T: AudioStream>(world: &mut World) {
    let world = world.cell();

    let mut audio_output = world.get_non_send_mut::<AudioOutput>().unwrap();
    let mut audio = world.get_resource_mut::<StreamedAudio<T>>().unwrap();

    audio_output.stream_audio(&mut *audio);
}
