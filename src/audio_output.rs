//! The internal audio systems and resource

use crate::audio::{AudioCommand, AudioCommandResult, AudioTween, PartialSoundSettings, map_tween};
use std::any::TypeId;

use crate::backend_settings::AudioSettings;
use crate::channel::dynamic::DynamicAudioChannels;
use crate::channel::typed::AudioChannel;
use crate::channel::{Channel, ChannelState};
use crate::instance::AudioInstance;
use crate::source::AudioSource;
use crate::{PlaybackState, SpatialAudioEmitter, TrackRegistry};
use bevy::asset::{Assets, Handle};
use bevy::ecs::change_detection::{NonSendMut, ResMut};
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{NonSend, Query, Res};
use bevy::ecs::world::{FromWorld, World};
use bevy::log::{error, warn};
use kira::backend::Backend;
use kira::track::TrackBuilder;
use kira::{AudioManager, Decibels, DefaultBackend, Panning, PlaybackRate, Value};
use std::collections::{HashMap, VecDeque};

/// Non-send resource that acts as audio output
///
/// This struct holds the [`AudioManager`] to play audio through. It also
/// keeps track of all audio instance handles and which sounds are playing in which channel.
pub struct AudioOutput<B: Backend = DefaultBackend> {
    pub(crate) manager: Option<AudioManager<B>>,
    instances: HashMap<Channel, Vec<Handle<AudioInstance>>>,
    channels: HashMap<Channel, ChannelState>,
    pub(crate) listener: Option<kira::listener::ListenerHandle>,
}

impl FromWorld for AudioOutput {
    fn from_world(world: &mut World) -> Self {
        let settings = world.remove_resource::<AudioSettings>().unwrap_or_default();
        let manager_result = AudioManager::new(settings.into());
        if let Err(ref setup_error) = manager_result {
            warn!("Failed to setup audio: {:?}", setup_error);
        }

        let mut manager = manager_result.ok();

        // Create the listener using the new manager
        let listener = manager.as_mut().and_then(|m| {
            let position = mint::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            let orientation = mint::Quaternion {
                v: mint::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                s: 1.0,
            };
            match m.add_listener(position, orientation) {
                Ok(handle) => Some(handle),
                Err(e) => {
                    warn!("Failed to create a listener for spatial audio: {:?}", e);
                    None
                }
            }
        });
        Self {
            manager,
            instances: HashMap::default(),
            channels: HashMap::default(),
            listener,
        }
    }
}

impl<B: Backend> AudioOutput<B> {
    fn stop(
        &mut self,
        channel: &Channel,
        audio_instances: &mut Assets<AudioInstance>,
        tween: &Option<AudioTween>,
    ) -> AudioCommandResult {
        if let Some(instances) = self.instances.get_mut(channel) {
            let tween = map_tween(tween);
            for instance in instances {
                if let Some(instance) = audio_instances.get_mut(instance.id()) {
                    instance.handle.stop(tween);
                }
            }
        }

        AudioCommandResult::Ok
    }

    fn pause(
        &mut self,
        channel: &Channel,
        audio_instances: &mut Assets<AudioInstance>,
        tween: &Option<AudioTween>,
    ) {
        if let Some(instance_handles) = self.instances.get_mut(channel) {
            let tween = map_tween(tween);
            for instance in instance_handles.iter_mut() {
                if let Some(instance) = audio_instances.get_mut(instance.id())
                    && kira::sound::PlaybackState::Playing == instance.handle.state()
                {
                    instance.handle.pause(tween);
                }
            }
        }
        if let Some(channel_state) = self.channels.get_mut(channel) {
            channel_state.paused = true;
        } else {
            let channel_state = ChannelState {
                paused: true,
                ..Default::default()
            };
            self.channels.insert(channel.clone(), channel_state);
        }
    }

    fn resume(
        &mut self,
        channel: &Channel,
        audio_instances: &mut Assets<AudioInstance>,
        tween: &Option<AudioTween>,
    ) {
        if let Some(instances) = self.instances.get_mut(channel) {
            let tween = map_tween(tween);
            for instance in instances.iter_mut() {
                if let Some(instance) = audio_instances.get_mut(instance.id())
                    && (instance.handle.state() == kira::sound::PlaybackState::Paused
                        || instance.handle.state() == kira::sound::PlaybackState::Pausing
                        || instance.handle.state() == kira::sound::PlaybackState::Stopping)
                {
                    instance.handle.resume(tween);
                }
            }
        }
        if let Some(channel_state) = self.channels.get_mut(channel) {
            channel_state.paused = false;
        } else {
            self.channels
                .insert(channel.clone(), ChannelState::default());
        }
    }

    fn set_volume(
        &mut self,
        channel: &Channel,
        audio_instances: &mut Assets<AudioInstance>,
        volume: Decibels,
        tween: &Option<AudioTween>,
    ) {
        if let Some(instances) = self.instances.get_mut(channel) {
            let tween = map_tween(tween);
            for instance in instances.iter_mut() {
                if let Some(instance) = audio_instances.get_mut(instance.id()) {
                    instance.handle.set_volume(volume, tween);
                }
            }
        }
        if let Some(channel_state) = self.channels.get_mut(channel) {
            channel_state.volume = volume;
        } else {
            let channel_state = ChannelState {
                volume,
                ..Default::default()
            };
            self.channels.insert(channel.clone(), channel_state);
        }
    }

    fn set_panning(
        &mut self,
        channel: &Channel,
        audio_instances: &mut Assets<AudioInstance>,
        panning: Panning,
        tween: &Option<AudioTween>,
    ) {
        if let Some(instances) = self.instances.get_mut(channel) {
            let tween = map_tween(tween);
            for instance in instances.iter_mut() {
                if let Some(instance) = audio_instances.get_mut(instance.id()) {
                    instance.handle.set_panning(panning, tween);
                }
            }
        }
        if let Some(channel_state) = self.channels.get_mut(channel) {
            channel_state.panning = panning;
        } else {
            let channel_state = ChannelState {
                panning,
                ..Default::default()
            };
            self.channels.insert(channel.clone(), channel_state);
        }
    }

    fn set_playback_rate(
        &mut self,
        channel: &Channel,
        audio_instances: &mut Assets<AudioInstance>,
        playback_rate: f64,
        tween: &Option<AudioTween>,
    ) {
        if let Some(instances) = self.instances.get_mut(channel) {
            let tween = map_tween(tween);
            for instance in instances.iter_mut() {
                if let Some(instance) = audio_instances.get_mut(instance.id()) {
                    instance.handle.set_playback_rate(playback_rate, tween);
                }
            }
        }
        if let Some(channel_state) = self.channels.get_mut(channel) {
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
        channel: &Channel,
        partial_sound_settings: &PartialSoundSettings,
        audio_source: &AudioSource,
        instance_handle: Handle<AudioInstance>,
        audio_instances: &mut Assets<AudioInstance>,
    ) -> AudioCommandResult {
        let mut sound = audio_source.sound.clone();
        if let Some(channel_state) = self.channels.get(channel) {
            channel_state.apply(&mut sound);
            // This is reverted after pausing the sound handle.
            // Otherwise the audio thread will start playing the sound before our pause command goes through.
            if channel_state.paused {
                sound.settings.playback_rate = Value::Fixed(PlaybackRate(0.0));
            }
        }
        if partial_sound_settings.paused {
            sound.settings.playback_rate = Value::Fixed(PlaybackRate(0.0));
        }
        partial_sound_settings.apply(&mut sound);
        let sound_handle = self.manager.as_mut().unwrap().play(sound);
        if let Err(error) = sound_handle {
            warn!("Failed to play sound due to {:?}", error);
            return AudioCommandResult::Ok;
        }
        let mut sound_handle = sound_handle.unwrap();
        if let Some(channel_state) = self.channels.get(channel)
            && channel_state.paused
        {
            sound_handle.pause(kira::Tween::default());
            let playback_rate = partial_sound_settings
                .playback_rate
                .unwrap_or(channel_state.playback_rate);
            sound_handle.set_playback_rate(playback_rate, kira::Tween::default());
        }
        if partial_sound_settings.paused {
            sound_handle.pause(kira::Tween::default());
            let playback_rate = partial_sound_settings.playback_rate.unwrap_or(1.0);
            sound_handle.set_playback_rate(playback_rate, kira::Tween::default());
        }
        let _ = audio_instances.insert(
            &instance_handle,
            AudioInstance {
                handle: sound_handle,
            },
        );
        if let Some(instance_states) = self.instances.get_mut(channel) {
            instance_states.push(instance_handle);
        } else {
            self.instances
                .insert(channel.clone(), vec![instance_handle]);
        }

        AudioCommandResult::Ok
    }
    pub(crate) fn play_audio_channel<T: Resource>(
        &mut self,
        channel: &AudioChannel<T>,
        track_registry: &mut TrackRegistry,
        audio_sources: &Assets<AudioSource>,
        audio_instances: &mut Assets<AudioInstance>,
        emitters: &mut Query<&mut SpatialAudioEmitter>,
    ) {
        if self.manager.is_none() {
            return;
        };

        let channel_id = Channel::Typed(TypeId::of::<T>());
        let mut commands = channel.commands.write();
        process_channel_commands(
            &channel_id,
            &mut commands,
            self,
            track_registry,
            audio_sources,
            audio_instances,
            emitters,
        );
    }

    pub(crate) fn play_dynamic_channels(
        &mut self,
        channels: &DynamicAudioChannels,
        track_registry: &mut TrackRegistry,
        audio_sources: &Assets<AudioSource>,
        audio_instances: &mut Assets<AudioInstance>,
        emitters: &mut Query<&mut SpatialAudioEmitter>,
    ) {
        if self.manager.is_none() {
            return;
        }

        for (key, channel) in channels.channels.iter() {
            let channel_id = Channel::Dynamic(key.clone());
            let mut commands = channel.commands.write();
            process_channel_commands(
                &channel_id,
                &mut commands,
                self,
                track_registry,
                audio_sources,
                audio_instances,
                emitters,
            );
        }
    }

    pub(crate) fn run_audio_command(
        &mut self,
        audio_command: &AudioCommand,
        track_registry: &mut TrackRegistry,
        audio_sources: &Assets<AudioSource>,
        audio_instances: &mut Assets<AudioInstance>,
        channel: &Channel,
        emitters: &mut Query<&mut SpatialAudioEmitter>,
    ) -> AudioCommandResult {
        match audio_command {
            AudioCommand::Play(play_args) => {
                if audio_sources.get(&play_args.source).is_none() {
                    return AudioCommandResult::Retry;
                }

                let source = audio_sources.get(&play_args.source).unwrap();
                let mut sound_data = source.sound.clone();
                play_args.settings.apply(&mut sound_data);

                // Determine which track to play on and get the resulting sound handle from Kira.
                let new_kira_handle = if let Some(emitter_entity) = play_args.settings.emitter {
                    // Play on a spatial emitter's track
                    emitters
                        .get_mut(emitter_entity)
                        .ok()
                        .and_then(|mut emitter| {
                            emitter
                                .track
                                .as_mut()
                                .and_then(|track| track.play(sound_data).ok())
                        })
                } else if let Channel::Typed(type_id) = channel
                    && !track_registry.handles.contains_key(type_id)
                    && let Some(manager) = self.manager.as_mut()
                {
                    if let Ok(handle) = manager.add_sub_track(TrackBuilder::new()) {
                        println!("new track");
                        track_registry.handles.insert(*type_id, handle);
                    }
                    track_registry
                        .handles
                        .get_mut(type_id)
                        .and_then(|track| track.play(sound_data).ok())
                } else {
                    // Play on the main track
                    self.manager.as_mut().and_then(|m| m.play(sound_data).ok())
                };

                if let Some(kira_handle) = new_kira_handle {
                    if let Err(error) = audio_instances.insert(
                        &play_args.instance_handle,
                        AudioInstance {
                            handle: kira_handle,
                        },
                    ) {
                        error!("Failed to insert audio instance: {error}");
                        return AudioCommandResult::Retry;
                    }

                    self.instances
                        .entry(channel.clone())
                        .or_default()
                        .push(play_args.instance_handle.clone());
                    AudioCommandResult::Ok
                } else {
                    AudioCommandResult::Retry
                }
            }
            AudioCommand::Stop(tween) => self.stop(channel, audio_instances, tween),
            AudioCommand::Pause(tween) => {
                self.pause(channel, audio_instances, tween);
                AudioCommandResult::Ok
            }
            AudioCommand::Resume(tween) => {
                self.resume(channel, audio_instances, tween);
                AudioCommandResult::Ok
            }
            AudioCommand::SetVolume(volume, tween) => {
                self.set_volume(channel, audio_instances, *volume, tween);
                AudioCommandResult::Ok
            }
            AudioCommand::SetPanning(panning, tween) => {
                self.set_panning(channel, audio_instances, *panning, tween);
                AudioCommandResult::Ok
            }
            AudioCommand::SetPlaybackRate(playback_rate, tween) => {
                self.set_playback_rate(channel, audio_instances, *playback_rate, tween);
                AudioCommandResult::Ok
            }
        }
    }

    pub(crate) fn cleanup_stopped_instances(&mut self, instances: &mut Assets<AudioInstance>) {
        for (_, handles) in self.instances.iter_mut() {
            handles.retain(|handle| {
                if let Some(instance) = instances.get(handle) {
                    instance.handle.state() != kira::sound::PlaybackState::Stopped
                } else {
                    false
                }
            });
        }
    }
}

pub(crate) fn play_audio_channel<T: Resource>(
    mut audio_output: NonSendMut<AudioOutput>,
    channel: Res<AudioChannel<T>>,
    mut track_registry: ResMut<TrackRegistry>,
    audio_sources: Option<Res<Assets<AudioSource>>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut emitters: Query<&mut SpatialAudioEmitter>,
) {
    if let Some(audio_sources) = audio_sources {
        audio_output.play_audio_channel(
            &channel,
            &mut track_registry,
            &audio_sources,
            &mut audio_instances,
            &mut emitters,
        );
    }
}

pub(crate) fn play_dynamic_channels(
    mut audio_output: NonSendMut<AudioOutput>,
    channels: Res<DynamicAudioChannels>,
    mut track_registry: ResMut<TrackRegistry>,
    audio_sources: Option<Res<Assets<AudioSource>>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut emitters: Query<&mut SpatialAudioEmitter>,
) {
    if let Some(audio_sources) = audio_sources {
        audio_output.play_dynamic_channels(
            &channels,
            &mut track_registry,
            &audio_sources,
            &mut audio_instances,
            &mut emitters,
        );
    }
}
pub(crate) fn cleanup_stopped_instances(
    mut audio_output: NonSendMut<AudioOutput>,
    mut instances: ResMut<Assets<AudioInstance>>,
) {
    audio_output.cleanup_stopped_instances(&mut instances);
}

pub(crate) fn update_instance_states<T: Resource>(
    audio_output: NonSend<AudioOutput>,
    audio_instances: Res<Assets<AudioInstance>>,
    mut channel: ResMut<AudioChannel<T>>,
) {
    if let Some(instances) = audio_output
        .instances
        .get(&Channel::Typed(TypeId::of::<T>()))
    {
        channel.states.clear();
        for instance_handle in instances.iter() {
            let state = audio_instances
                .get(instance_handle)
                .map(|instance| instance.state())
                .unwrap_or(PlaybackState::Stopped);
            channel.states.insert(instance_handle.id(), state);
        }
    }
}
/// Contains the shared logic for processing a queue of audio commands for a specific channel.
fn process_channel_commands<B: Backend>(
    channel_id: &Channel,
    commands: &mut VecDeque<AudioCommand>,
    audio_output: &mut AudioOutput<B>,
    track_registry: &mut TrackRegistry,
    audio_sources: &Assets<AudioSource>,
    audio_instances: &mut Assets<AudioInstance>,
    emitters: &mut Query<&mut SpatialAudioEmitter>,
) {
    let mut still_queued = VecDeque::new();

    for audio_command in commands.drain(..).rev() {
        let result = audio_output.run_audio_command(
            &audio_command,
            track_registry,
            audio_sources,
            audio_instances,
            channel_id,
            emitters,
        );
        if let AudioCommandResult::Retry = result {
            still_queued.push_front(audio_command);
        }
    }
    *commands = still_queued;
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;

    use super::*;
    use crate::{Audio, AudioControl, AudioPlugin};
    use bevy::asset::AssetPlugin;
    use bevy::prelude::*;
    use uuid::Uuid;

    // Helper to create a minimal app for testing
    fn setup_test_app() -> App {
        let mut app = App::new();
        // The tests need the full plugin to initialize resources like AudioOutput, TrackRegistry, etc.
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), AudioPlugin));
        app
    }

    #[test]
    fn keeps_order_of_commands_to_retry() {
        let mut app = setup_test_app();

        let audio_handle_one: Handle<AudioSource> = Handle::Uuid(Uuid::new_v4(), PhantomData);
        let audio_handle_two: Handle<AudioSource> = Handle::Uuid(Uuid::new_v4(), PhantomData);

        // Get the Audio resource from the world and queue the commands
        let audio = app.world().resource::<Audio>();
        audio.play(audio_handle_one.clone());
        audio.play(audio_handle_two.clone());

        // Run the systems. Because the assets are not loaded in Assets<AudioSource>,
        // the play commands should remain in the queue.
        app.update();

        {
            let audio = app.world().resource::<Audio>();
            let commands = audio.commands.read();

            let command_one = commands.back().unwrap();
            match command_one {
                AudioCommand::Play(settings) => {
                    assert_eq!(settings.source.id(), audio_handle_one.id())
                }
                _ => panic!("Wrong audio command"),
            }

            let command_two = commands.front().unwrap();
            match command_two {
                AudioCommand::Play(settings) => {
                    assert_eq!(settings.source.id(), audio_handle_two.id())
                }
                _ => panic!("Wrong audio command"),
            }
        }
        app.update();
        {
            let audio = app.world().resource::<Audio>();
            let commands = audio.commands.read();

            let command_one = commands.back().unwrap();
            match command_one {
                AudioCommand::Play(settings) => {
                    assert_eq!(settings.source.id(), audio_handle_one.id())
                }
                _ => panic!("Wrong audio command"),
            }

            let command_two = commands.front().unwrap();
            match command_two {
                AudioCommand::Play(settings) => {
                    assert_eq!(settings.source.id(), audio_handle_two.id())
                }
                _ => panic!("Wrong audio command"),
            }
        }
    }
    #[test]
    fn stop_command_is_queued() {
        let app = setup_test_app();

        let audio_handle_one: Handle<AudioSource> = Handle::Uuid(Uuid::new_v4(), PhantomData);
        let audio_handle_two: Handle<AudioSource> = Handle::Uuid(Uuid::new_v4(), PhantomData);

        let audio = app.world().resource::<Audio>();
        audio.play(audio_handle_one.clone());
        audio.stop();
        audio.play(audio_handle_two.clone());

        // Check the command queue state BEFORE the systems run.
        let mut commands = audio.commands.write();
        assert_eq!(commands.len(), 3);

        // Test that the commands are in the correct LIFO order
        match commands.pop_front().unwrap() {
            AudioCommand::Play(s) => assert_eq!(s.source, audio_handle_two),
            _ => panic!("Expected Play command"),
        }
        match commands.pop_front().unwrap() {
            AudioCommand::Stop(_) => {} // Correct
            _ => panic!("Expected Stop command"),
        }
        match commands.pop_front().unwrap() {
            AudioCommand::Play(s) => assert_eq!(s.source, audio_handle_one),
            _ => panic!("Expected Play command"),
        }
    }
}
