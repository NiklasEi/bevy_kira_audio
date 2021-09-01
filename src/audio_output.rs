use crate::audio::{Audio, AudioCommand, AudioCommandResult, PlayAudioSettings};
use bevy::prelude::*;
use bevy_utils::tracing::warn;

use crate::channel::AudioChannel;
use crate::source::AudioSource;
use crate::stream::{InternalAudioStream, StreamCommands, StreamedAudio};
use crate::AudioStream;
use kira::arrangement::handle::ArrangementHandle;
use kira::arrangement::{Arrangement, ArrangementSettings, SoundClip};
use kira::audio_stream::AudioStreamId;
use kira::instance::handle::InstanceHandle;
use kira::instance::{
    InstanceState, PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings,
};
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::mixer::TrackIndex;
use kira::sound::handle::SoundHandle;
use kira::CommandError;
use std::collections::HashMap;

/// Non-send resource that acts as audio output
///
/// This struct holds the [kira::manager::AudioManager] to play audio through. It also
/// keeps track of all audio instance handles and which sounds are playing in which channel.
pub struct AudioOutput {
    manager: Option<AudioManager>,
    sounds: HashMap<Handle<AudioSource>, SoundHandle>,
    arrangements: HashMap<PlayAudioSettings, ArrangementHandle>,
    streams: HashMap<AudioChannel, Vec<AudioStreamId>>,
    instances: HashMap<AudioChannel, Vec<InstanceHandle>>,
    channels: HashMap<AudioChannel, ChannelState>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        let manager = AudioManager::new(AudioManagerSettings::default());
        if let Err(ref setup_error) = manager {
            warn!("Failed to setup audio: {:?}", setup_error);
        }

        Self {
            manager: manager.ok(),
            sounds: HashMap::default(),
            arrangements: HashMap::default(),
            streams: HashMap::default(),
            instances: HashMap::default(),
            channels: HashMap::default(),
        }
    }
}

impl AudioOutput {
    fn create_or_get_sound(
        &mut self,
        audio_source: &AudioSource,
        audio_source_handle: Handle<AudioSource>,
    ) -> SoundHandle {
        if let Some(handle) = self.sounds.get(&audio_source_handle) {
            return handle.clone();
        }

        let sound = audio_source.sound.clone();
        let manager = self.manager.as_mut().unwrap();
        let handle = manager
            .add_sound(sound)
            .expect("Failed to add sound to the AudioManager");
        self.sounds.insert(audio_source_handle, handle.clone());
        handle
    }

    fn play_arrangement(
        &mut self,
        mut arrangement_handle: ArrangementHandle,
        channel: &AudioChannel,
    ) -> AudioCommandResult {
        let play_result = arrangement_handle.play(Default::default());
        if let Err(error) = play_result {
            return match error {
                CommandError::CommandQueueFull => AudioCommandResult::Retry,
                _ => {
                    println!("Failed to play arrangement: {:?}", error);
                    AudioCommandResult::Ok
                }
            };
        }
        let mut instance_handle = play_result.unwrap();
        if let Some(channel_state) = self.channels.get(channel) {
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
        if let Some(instance_handles) = self.instances.get_mut(channel) {
            instance_handles.push(instance_handle);
        } else {
            self.instances
                .insert(channel.clone(), vec![instance_handle]);
        }

        AudioCommandResult::Ok
    }

    fn create_arrangement(&mut self, sound_handle: &SoundHandle) -> ArrangementHandle {
        let mut arrangement = Arrangement::new(ArrangementSettings::new().cooldown(0.0));
        arrangement.add_clip(SoundClip::new(sound_handle, 0.0));
        let manager = self.manager.as_mut().unwrap();
        manager
            .add_arrangement(arrangement)
            .expect("Failed to add arrangement to the AudioManager")
    }

    fn create_looped_arrangement(&mut self, sound_handle: &SoundHandle) -> ArrangementHandle {
        let arrangement = Arrangement::new_loop(sound_handle, Default::default());
        let manager = self.manager.as_mut().unwrap();
        manager
            .add_arrangement(arrangement)
            .expect("Failed to add arrangement to the AudioManager")
    }

    fn create_looped_arrangement_with_intro(
        &mut self,
        intro_sound_handle: &SoundHandle,
        loop_sound_handle: &SoundHandle,
    ) -> ArrangementHandle {
        let arrangement = Arrangement::new_loop_with_intro(
            intro_sound_handle,
            loop_sound_handle,
            Default::default(),
        );
        let manager = self.manager.as_mut().unwrap();
        manager
            .add_arrangement(arrangement)
            .expect("Failed to add arrangement to the AudioManager")
    }

    fn stop(&mut self, channel: &AudioChannel) -> AudioCommandResult {
        if let Some(instances) = self.instances.get_mut(channel) {
            while !instances.is_empty() {
                let mut instance = instances.remove(0);
                if let Err(error) = instance.stop(StopInstanceSettings::default()) {
                    match error {
                        CommandError::CommandQueueFull => {
                            instances.push(instance);
                            return AudioCommandResult::Retry;
                        }
                        _ => {
                            println!("Failed to stop instance: {:?}", error);
                        }
                    }
                }
            }
        }

        AudioCommandResult::Ok
    }

    fn pause(&mut self, channel: &AudioChannel) {
        if let Some(instances) = self.instances.get_mut(channel) {
            for instance in instances.iter_mut() {
                if InstanceState::Playing == instance.state() {
                    if let Err(error) = instance.pause(PauseInstanceSettings::default()) {
                        println!("Failed to pause instance: {:?}", error);
                    }
                }
            }
        }
    }

    fn resume(&mut self, channel: &AudioChannel) {
        if let Some(instances) = self.instances.get_mut(channel) {
            for instance in instances.iter_mut() {
                if let InstanceState::Paused(_position) = instance.state() {
                    if let Err(error) = instance.resume(ResumeInstanceSettings::default()) {
                        println!("Failed to resume instance: {:?}", error);
                    }
                }
            }
        }
    }

    fn set_volume(&mut self, channel: &AudioChannel, volume: f64) {
        if let Some(instances) = self.instances.get_mut(channel) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.set_volume(volume) {
                    println!("Failed to set volume for instance: {:?}", error);
                }
            }
        }
        if let Some(mut channel_state) = self.channels.get_mut(channel) {
            channel_state.volume = volume;
        } else {
            let channel_state = ChannelState {
                volume,
                ..Default::default()
            };
            self.channels.insert(channel.clone(), channel_state);
        }
    }

    fn set_panning(&mut self, channel: &AudioChannel, panning: f64) {
        if let Some(instances) = self.instances.get_mut(channel) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.set_panning(panning) {
                    println!("Failed to set panning for instance: {:?}", error);
                }
            }
        }
        if let Some(mut channel_state) = self.channels.get_mut(channel) {
            channel_state.panning = panning;
        } else {
            let channel_state = ChannelState {
                panning,
                ..Default::default()
            };
            self.channels.insert(channel.clone(), channel_state);
        }
    }

    fn set_playback_rate(&mut self, channel: &AudioChannel, playback_rate: f64) {
        if let Some(instances) = self.instances.get_mut(channel) {
            for instance in instances.iter_mut() {
                if let Err(error) = instance.set_playback_rate(playback_rate) {
                    println!("Failed to set playback rate for instance: {:?}", error);
                }
            }
        }
        if let Some(mut channel_state) = self.channels.get_mut(channel) {
            channel_state.playback_rate = playback_rate;
        } else {
            let channel_state = ChannelState {
                playback_rate,
                ..Default::default()
            };
            self.channels.insert(channel.clone(), channel_state);
        }
    }

    fn play(
        &mut self,
        channel: &AudioChannel,
        play_settings: &PlayAudioSettings,
        audio_source: &AudioSource,
        intro_audio_source: Option<&AudioSource>,
    ) -> AudioCommandResult {
        if self.arrangements.contains_key(play_settings) {
            self.play_arrangement(
                self.arrangements.get(play_settings).unwrap().clone(),
                channel,
            )
        } else {
            let sound_handle = self.create_or_get_sound(audio_source, play_settings.source.clone());
            let intro_handle = intro_audio_source
                .zip(play_settings.intro_source.as_ref())
                .map(|(audio_source, handle)| {
                    self.create_or_get_sound(audio_source, handle.clone())
                });
            let arrangement_handle = match (play_settings.looped, intro_handle) {
                (true, Some(intro_source)) => {
                    self.create_looped_arrangement_with_intro(&intro_source, &sound_handle)
                }
                (true, None) => self.create_looped_arrangement(&sound_handle),
                (false, _) => self.create_arrangement(&sound_handle),
            };

            self.arrangements
                .insert(play_settings.clone(), arrangement_handle.clone());
            self.play_arrangement(arrangement_handle, channel)
        }
    }

    pub(crate) fn run_queued_audio_commands(
        &mut self,
        audio_sources: &Assets<AudioSource>,
        audio: &mut Audio,
    ) {
        if self.manager.is_none() {
            return;
        }
        let mut commands = audio.commands.write();
        let len = commands.len();
        let mut i = 0;
        while i < len {
            let (audio_command, channel) = commands.pop_back().unwrap();
            let result = match &audio_command {
                AudioCommand::Play(play_settings) => {
                    let intro_audio = play_settings
                        .intro_source
                        .as_ref()
                        .and_then(|source| audio_sources.get(source));
                    if let Some(audio_source) = audio_sources.get(&play_settings.source) {
                        if intro_audio.is_some() == play_settings.intro_source.is_some() {
                            self.play(&channel, play_settings, audio_source, intro_audio)
                        } else {
                            // Intro audio source hasn't loaded yet. Add it back to the queue
                            AudioCommandResult::Retry
                        }
                    } else {
                        // audio source hasn't loaded yet. Add it back to the queue
                        AudioCommandResult::Retry
                    }
                }
                AudioCommand::Stop => self.stop(&channel),
                AudioCommand::Pause => {
                    self.pause(&channel);
                    AudioCommandResult::Ok
                }
                AudioCommand::Resume => {
                    self.resume(&channel);
                    AudioCommandResult::Ok
                }
                AudioCommand::SetVolume(volume) => {
                    self.set_volume(&channel, *volume as f64);
                    AudioCommandResult::Ok
                }
                AudioCommand::SetPanning(panning) => {
                    self.set_panning(&channel, *panning as f64);
                    AudioCommandResult::Ok
                }
                AudioCommand::SetPlaybackRate(playback_rate) => {
                    self.set_playback_rate(&channel, *playback_rate as f64);
                    AudioCommandResult::Ok
                }
            };
            if let AudioCommandResult::Retry = result {
                commands.push_front((audio_command, channel))
            }
            i += 1;
        }
    }

    pub(crate) fn cleanup_stopped_instances(&mut self) {
        for (_, instances) in self.instances.iter_mut() {
            instances.retain(|instance| instance.state() != InstanceState::Stopped);
        }
    }

    fn start_stream<T: kira::audio_stream::AudioStream>(
        &mut self,
        stream: T,
        channel: AudioChannel,
    ) {
        let manager = self.manager.as_mut().unwrap();
        let stream_id = manager
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
            let manager = self.manager.as_mut().unwrap();
            for stream_id in stream_ids.drain(..) {
                if let Err(error) = manager.remove_stream(stream_id) {
                    println!("Failed to stop stream: {:?}", error);
                }
            }
        }
    }

    pub(crate) fn stream_audio<T: AudioStream>(&mut self, audio: &mut StreamedAudio<T>) {
        if self.manager.is_none() {
            return;
        }
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
        audio_output.cleanup_stopped_instances();
    };
}

pub fn stream_audio_system<T: AudioStream>(world: &mut World) {
    let world = world.cell();

    let mut audio_output = world.get_non_send_mut::<AudioOutput>().unwrap();
    let mut audio = world.get_resource_mut::<StreamedAudio<T>>().unwrap();

    audio_output.stream_audio(&mut *audio);
}
