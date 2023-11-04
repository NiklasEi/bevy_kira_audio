//! The internal audio systems and resource

use crate::audio::{map_tween, AudioCommand, AudioCommandResult, AudioTween, PartialSoundSettings};
use std::any::TypeId;

use crate::backend_settings::AudioSettings;
use crate::channel::dynamic::DynamicAudioChannels;
use crate::channel::typed::AudioChannel;
use crate::channel::{Channel, ChannelState};
use crate::instance::AudioInstance;
use crate::source::AudioSource;
use crate::PlaybackState;
use bevy::asset::{Assets, Handle};
use bevy::ecs::change_detection::{NonSendMut, ResMut};
use bevy::ecs::system::{NonSend, Res, Resource};
use bevy::ecs::world::{FromWorld, World};
use bevy::log::{error, warn};
use kira::manager::backend::{Backend, DefaultBackend};
use kira::manager::AudioManager;
use kira::{sound::PlaybackRate, CommandError, Volume};
use std::collections::HashMap;

/// Non-send resource that acts as audio output
///
/// This struct holds the [`AudioManager`] to play audio through. It also
/// keeps track of all audio instance handles and which sounds are playing in which channel.
pub(crate) struct AudioOutput<B: Backend = DefaultBackend> {
    manager: Option<AudioManager<B>>,
    instances: HashMap<Channel, Vec<Handle<AudioInstance>>>,
    channels: HashMap<Channel, ChannelState>,
}

impl FromWorld for AudioOutput {
    fn from_world(world: &mut World) -> Self {
        let settings = world.remove_resource::<AudioSettings>().unwrap_or_default();
        let manager = AudioManager::new(settings.into());
        if let Err(ref setup_error) = manager {
            warn!("Failed to setup audio: {:?}", setup_error);
        }

        Self {
            manager: manager.ok(),
            instances: HashMap::default(),
            channels: HashMap::default(),
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
                    match instance.handle.stop(tween) {
                        Err(CommandError::CommandQueueFull) => {
                            return AudioCommandResult::Retry;
                        }
                        Err(error) => {
                            error!("Failed to stop instance: {:?}", error);
                        }
                        _ => (),
                    }
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
                if let Some(instance) = audio_instances.get_mut(instance.id()) {
                    if kira::sound::PlaybackState::Playing == instance.handle.state() {
                        if let Err(error) = instance.handle.pause(tween) {
                            error!("Failed to pause instance: {:?}", error);
                        }
                    }
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
                if let Some(instance) = audio_instances.get_mut(instance.id()) {
                    if instance.handle.state() == kira::sound::PlaybackState::Paused
                        || instance.handle.state() == kira::sound::PlaybackState::Pausing
                        || instance.handle.state() == kira::sound::PlaybackState::Stopping
                    {
                        if let Err(error) = instance.handle.resume(tween) {
                            error!("Failed to resume instance: {:?}", error);
                        }
                    }
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
        volume: Volume,
        tween: &Option<AudioTween>,
    ) {
        if let Some(instances) = self.instances.get_mut(channel) {
            let tween = map_tween(tween);
            for instance in instances.iter_mut() {
                if let Some(instance) = audio_instances.get_mut(instance.id()) {
                    if let Err(error) = instance.handle.set_volume(volume, tween) {
                        error!("Failed to set volume for instance: {:?}", error);
                    }
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
        panning: f64,
        tween: &Option<AudioTween>,
    ) {
        if let Some(instances) = self.instances.get_mut(channel) {
            let tween = map_tween(tween);
            for instance in instances.iter_mut() {
                if let Some(instance) = audio_instances.get_mut(instance.id()) {
                    if let Err(error) = instance.handle.set_panning(panning, tween) {
                        error!("Failed to set panning for instance: {:?}", error);
                    }
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
                    if let Err(error) = instance.handle.set_playback_rate(playback_rate, tween) {
                        error!("Failed to set playback rate for instance: {:?}", error);
                    }
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
                sound.settings.playback_rate = kira::tween::Value::Fixed(PlaybackRate::Factor(0.0));
            }
        }
        if partial_sound_settings.paused {
            sound.settings.playback_rate = kira::tween::Value::Fixed(PlaybackRate::Factor(0.0));
        }
        partial_sound_settings.apply(&mut sound);
        let sound_handle = self.manager.as_mut().unwrap().play(sound);
        if let Err(error) = sound_handle {
            warn!("Failed to play sound due to {:?}", error);
            return AudioCommandResult::Ok;
        }
        let mut sound_handle = sound_handle.unwrap();
        if let Some(channel_state) = self.channels.get(channel) {
            if channel_state.paused {
                if let Err(error) = sound_handle.pause(kira::tween::Tween::default()) {
                    warn!(
                        "Failed to pause instance (channel was paused) due to {:?}",
                        error
                    );
                }
                let playback_rate = partial_sound_settings
                    .playback_rate
                    .unwrap_or(channel_state.playback_rate);
                if let Err(error) =
                    sound_handle.set_playback_rate(playback_rate, kira::tween::Tween::default())
                {
                    error!("Failed to set playback rate for instance: {:?}", error);
                }
            }
        }
        if partial_sound_settings.paused {
            if let Err(error) = sound_handle.pause(kira::tween::Tween::default()) {
                warn!("Failed to pause instance due to {:?}", error);
            }
            let playback_rate = partial_sound_settings.playback_rate.unwrap_or(1.0);
            if let Err(error) =
                sound_handle.set_playback_rate(playback_rate, kira::tween::Tween::default())
            {
                error!("Failed to set playback rate for instance: {:?}", error);
            }
        }
        audio_instances.insert(
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

    pub(crate) fn play_channel<T: Resource>(
        &mut self,
        audio_sources: &Assets<AudioSource>,
        channel: &AudioChannel<T>,
        audio_instances: &mut Assets<AudioInstance>,
    ) {
        if self.manager.is_none() {
            return;
        }
        let mut commands = channel.commands.write();
        let len = commands.len();
        let channel_id = TypeId::of::<T>();
        let channel = Channel::Typed(channel_id);
        let mut commands_to_retry = vec![];
        let mut i = 0;
        while i < len {
            let audio_command = commands.pop_back().unwrap();
            let result =
                self.run_audio_command(&audio_command, audio_sources, audio_instances, &channel);
            if let AudioCommand::Stop(_) = audio_command {
                commands_to_retry.clear();
            }
            if let AudioCommandResult::Retry = result {
                commands_to_retry.push(audio_command);
            }
            i += 1;
        }
        commands_to_retry
            .drain(..)
            .for_each(|command| commands.push_front(command));
    }

    pub(crate) fn play_dynamic_channels(
        &mut self,
        audio_sources: &Assets<AudioSource>,
        channels: &DynamicAudioChannels,
        audio_instances: &mut Assets<AudioInstance>,
    ) {
        if self.manager.is_none() {
            return;
        }
        for (key, channel) in channels.channels.iter() {
            let mut commands = channel.commands.write();
            let len = commands.len();
            let channel = Channel::Dynamic(key.clone());
            let mut i = 0;
            while i < len {
                let audio_command = commands.pop_back().unwrap();
                let result = self.run_audio_command(
                    &audio_command,
                    audio_sources,
                    audio_instances,
                    &channel,
                );
                if let AudioCommandResult::Retry = result {
                    commands.push_front(audio_command);
                }
                i += 1;
            }
        }
    }

    pub(crate) fn run_audio_command(
        &mut self,
        audio_command: &AudioCommand,
        audio_sources: &Assets<AudioSource>,
        audio_instances: &mut Assets<AudioInstance>,
        channel: &Channel,
    ) -> AudioCommandResult {
        match audio_command {
            AudioCommand::Play(play_args) => {
                if let Some(audio_source) = audio_sources.get(&play_args.source) {
                    self.play(
                        channel,
                        &play_args.settings,
                        audio_source,
                        play_args.instance_handle.clone(),
                        audio_instances,
                    )
                } else {
                    // audio source hasn't loaded yet. Add it back to the queue
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

pub(crate) fn play_dynamic_channels(
    mut audio_output: NonSendMut<AudioOutput>,
    channels: Res<DynamicAudioChannels>,
    audio_sources: Option<Res<Assets<AudioSource>>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Some(audio_sources) = audio_sources {
        audio_output.play_dynamic_channels(&audio_sources, &channels, &mut audio_instances);
    };
}

pub(crate) fn play_audio_channel<T: Resource>(
    mut audio_output: NonSendMut<AudioOutput>,
    channel: Res<AudioChannel<T>>,
    audio_sources: Option<Res<Assets<AudioSource>>>,
    mut instances: ResMut<Assets<AudioInstance>>,
) {
    if let Some(audio_sources) = audio_sources {
        audio_output.play_channel(&audio_sources, &channel, &mut instances);
    };
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::channel::AudioControl;
    use crate::{Audio, AudioPlugin};
    use bevy::asset::{AssetId, AssetPlugin};
    use bevy::prelude::*;
    use bevy::utils::Uuid;
    use kira::manager::backend::mock::MockBackend;
    use kira::manager::AudioManagerSettings;

    #[test]
    fn keeps_order_of_commands_to_retry() {
        // we only need this app to conveniently get a assets collection for `AudioSource`...
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), AudioPlugin));
        let audio_source_assets = app.world.remove_resource::<Assets<AudioSource>>().unwrap();
        let mut audio_instance_assets = app
            .world
            .remove_resource::<Assets<AudioInstance>>()
            .unwrap();

        let mut audio_output = AudioOutput {
            manager: AudioManager::new(AudioManagerSettings::<MockBackend>::default()).ok(),
            instances: HashMap::default(),
            channels: HashMap::default(),
        };
        let audio_handle_one: Handle<AudioSource> = Handle::<AudioSource>::Weak(AssetId::Uuid {
            uuid: Uuid::from_u128(1758302748397294),
        });
        let audio_handle_two: Handle<AudioSource> = Handle::<AudioSource>::Weak(AssetId::Uuid {
            uuid: Uuid::from_u128(2537024739048739),
        });

        let channel = AudioChannel::<Audio>::default();
        channel.play(audio_handle_one.clone());
        channel.play(audio_handle_two.clone());

        audio_output.play_channel(&audio_source_assets, &channel, &mut audio_instance_assets);

        let command_one = channel.commands.write().pop_back().unwrap();
        match command_one {
            AudioCommand::Play(settings) => {
                assert_eq!(settings.source.id(), audio_handle_one.id())
            }
            _ => panic!("Wrong audio command"),
        }
        let command_two = channel.commands.write().pop_back().unwrap();
        match command_two {
            AudioCommand::Play(settings) => {
                assert_eq!(settings.source.id(), audio_handle_two.id())
            }
            _ => panic!("Wrong audio command"),
        }
    }

    #[test]
    fn stop_command_removes_previous_play_commands() {
        // we only need this app to conveniently get a assets collection for `AudioSource`...
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), AudioPlugin));
        let audio_source_assets = app.world.remove_resource::<Assets<AudioSource>>().unwrap();
        let mut audio_instance_assets = app
            .world
            .remove_resource::<Assets<AudioInstance>>()
            .unwrap();

        let mut audio_output = AudioOutput {
            manager: AudioManager::new(AudioManagerSettings::<MockBackend>::default()).ok(),
            instances: HashMap::default(),
            channels: HashMap::default(),
        };
        let audio_handle_one: Handle<AudioSource> = Handle::<AudioSource>::Weak(AssetId::Uuid {
            uuid: Uuid::from_u128(13290473942075938),
        });
        let audio_handle_two: Handle<AudioSource> = Handle::<AudioSource>::Weak(AssetId::Uuid {
            uuid: Uuid::from_u128(243290473942075938),
        });

        let channel = AudioChannel::<Audio>::default();
        channel.play(audio_handle_one);
        channel.stop();
        channel.play(audio_handle_two.clone());

        audio_output.play_channel(&audio_source_assets, &channel, &mut audio_instance_assets);

        let command = channel.commands.write().pop_back().unwrap();
        match command {
            AudioCommand::Play(settings) => {
                assert_eq!(settings.source.id(), audio_handle_two.id())
            }
            _ => panic!("Wrong audio command"),
        }
        assert!(channel.commands.write().pop_back().is_none());
    }
}
