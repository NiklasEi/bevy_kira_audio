//! The internal audio systems and resource

use crate::audio::{AudioCommand, AudioCommandResult, AudioTween, PartialSoundSettings, map_tween};
use std::any::TypeId;

use crate::PlaybackState;
use crate::backend_settings::AudioSettings;
use crate::channel::dynamic::DynamicAudioChannels;
use crate::channel::typed::AudioChannel;
use crate::channel::{Channel, ChannelState};
use crate::effect::AudioTrack;
use crate::instance::AudioInstance;
use crate::instance_track::InstanceTrack;
use crate::source::AudioSource;
use bevy::asset::{Assets, Handle};
use bevy::ecs::change_detection::{NonSendMut, ResMut};
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{NonSend, Res};
use bevy::ecs::world::{FromWorld, World};
use bevy::log::warn;
use kira::ResourceLimitReached;
use kira::backend::{Backend, DefaultBackend};
use kira::track::{TrackBuilder, TrackHandle};
use kira::{AudioManager, Panning};
use kira::{Decibels, PlaybackRate};
use std::collections::HashMap;

/// Non-send resource that acts as audio output
///
/// This struct holds the [`AudioManager`] to play audio through. It also
/// keeps track of all audio instance handles and which sounds are playing in which channel.
pub(crate) struct AudioOutput<B: Backend = DefaultBackend> {
    manager: Option<AudioManager<B>>,
    instances: HashMap<Channel, Vec<Handle<AudioInstance>>>,
    channels: HashMap<Channel, ChannelState>,
    channel_tracks: HashMap<Channel, TrackHandle>,
    instance_tracks: Vec<InstanceTrack>,
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
            channel_tracks: HashMap::default(),
            instance_tracks: Vec::new(),
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
                if let Some(mut instance) = audio_instances.get_mut(instance.id()) {
                    instance.stop_with(tween);
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
                if let Some(mut instance) = audio_instances.get_mut(instance.id())
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
                if let Some(mut instance) = audio_instances.get_mut(instance.id())
                    && (instance.handle.state() == kira::sound::PlaybackState::Paused
                        || instance.handle.state() == kira::sound::PlaybackState::Pausing
                        || instance.handle.state() == kira::sound::PlaybackState::Stopping)
                {
                    instance.resume_with(tween);
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
                if let Some(mut instance) = audio_instances.get_mut(instance.id()) {
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
                if let Some(mut instance) = audio_instances.get_mut(instance.id()) {
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
                if let Some(mut instance) = audio_instances.get_mut(instance.id()) {
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
        mut partial_sound_settings: PartialSoundSettings,
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
                sound.settings.playback_rate = kira::Value::Fixed(PlaybackRate(0.0));
            }
        }
        if partial_sound_settings.paused {
            sound.settings.playback_rate = kira::Value::Fixed(PlaybackRate(0.0));
        }
        partial_sound_settings.apply(&mut sound);

        let instance_track = partial_sound_settings.track.take().and_then(|track| {
            InstanceTrack::add(*track, partial_sound_settings.effect_tail, |track| {
                self.add_instance_track(channel, track)
            })
            .inspect_err(|error| {
                warn!(
                    "Failed to create a track for the sound's effects, playing it without them: \
                     {error:?}"
                );
            })
            .ok()
        });

        let (sound_handle, effects) = if let Some((mut track, effects)) = instance_track {
            // Per-instance effects: play on the sub-track of this instance
            let result = track.handle.play(sound);
            if result.is_ok() {
                self.instance_tracks.push(track);
            }
            (result, Some(effects))
        } else if let Some(track_handle) = self.channel_tracks.get_mut(channel) {
            // Channel-level effects: play on the channel's sub-track
            (track_handle.play(sound), None)
        } else {
            // No effects: play on the main track
            (self.manager.as_mut().unwrap().play(sound), None)
        };

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
                effects,
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
            let is_stop = matches!(audio_command, AudioCommand::Stop(_));
            let result =
                self.run_audio_command(audio_command, audio_sources, audio_instances, &channel);
            if is_stop {
                commands_to_retry.clear();
            }
            if let AudioCommandResult::Retry(audio_command) = result {
                commands_to_retry.push(*audio_command);
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
                let result =
                    self.run_audio_command(audio_command, audio_sources, audio_instances, &channel);
                if let AudioCommandResult::Retry(audio_command) = result {
                    commands.push_front(*audio_command);
                }
                i += 1;
            }
        }
    }

    pub(crate) fn run_audio_command(
        &mut self,
        audio_command: AudioCommand,
        audio_sources: &Assets<AudioSource>,
        audio_instances: &mut Assets<AudioInstance>,
        channel: &Channel,
    ) -> AudioCommandResult {
        match audio_command {
            AudioCommand::Play(play_args) => {
                let Some(audio_source) = audio_sources.get(&play_args.source) else {
                    // audio source hasn't loaded yet. Add it back to the queue
                    return AudioCommandResult::Retry(Box::new(AudioCommand::Play(play_args)));
                };
                self.play(
                    channel,
                    play_args.settings,
                    audio_source,
                    play_args.instance_handle,
                    audio_instances,
                )
            }
            AudioCommand::Stop(tween) => self.stop(channel, audio_instances, &tween),
            AudioCommand::Pause(tween) => {
                self.pause(channel, audio_instances, &tween);
                AudioCommandResult::Ok
            }
            AudioCommand::Resume(tween) => {
                self.resume(channel, audio_instances, &tween);
                AudioCommandResult::Ok
            }
            AudioCommand::SetVolume(volume, tween) => {
                self.set_volume(channel, audio_instances, volume, &tween);
                AudioCommandResult::Ok
            }
            AudioCommand::SetPanning(panning, tween) => {
                self.set_panning(channel, audio_instances, panning, &tween);
                AudioCommandResult::Ok
            }
            AudioCommand::SetPlaybackRate(playback_rate, tween) => {
                self.set_playback_rate(channel, audio_instances, playback_rate, &tween);
                AudioCommandResult::Ok
            }
        }
    }

    /// Add the sub-track carrying one sound's own effects.
    ///
    /// It goes under the channel's track if there is one, so the channel's effects run after the
    /// sound's.
    fn add_instance_track(
        &mut self,
        channel: &Channel,
        track: TrackBuilder,
    ) -> Result<TrackHandle, ResourceLimitReached> {
        if let Some(channel_track) = self.channel_tracks.get_mut(channel) {
            channel_track.add_sub_track(track)
        } else {
            self.manager.as_mut().unwrap().add_sub_track(track)
        }
    }

    pub(crate) fn has_channel_track(&self, channel: &Channel) -> bool {
        self.channel_tracks.contains_key(channel)
    }

    pub(crate) fn create_channel_track(&mut self, channel: Channel, track: AudioTrack) {
        if let Some(manager) = self.manager.as_mut() {
            match manager.add_sub_track(track.into_inner()) {
                Ok(track_handle) => {
                    self.channel_tracks.insert(channel, track_handle);
                }
                Err(error) => {
                    warn!("Failed to create channel sub-track: {:?}", error);
                }
            }
        }
    }

    pub(crate) fn cleanup_stopped_instances(&mut self, instances: &mut Assets<AudioInstance>) {
        for handles in self.instances.values_mut() {
            handles.retain(|handle| {
                instances.get(handle).is_some_and(|instance| {
                    instance.handle.state() != kira::sound::PlaybackState::Stopped
                })
            });
        }

        // Dropping the handle is what removes the sub-track from kira's audio graph.
        self.instance_tracks.retain(InstanceTrack::keep_alive);
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
    use std::marker::PhantomData;

    use super::*;
    use crate::channel::AudioControl;
    use crate::effect::{AudioEffect, Effect, EffectTail, FilterBuilder, Info, ReverbBuilder};
    use crate::{Audio, AudioPlugin, PlayAudioCommand};
    use bevy::asset::AssetPlugin;
    use bevy::prelude::*;
    use kira::AudioManagerSettings;
    use kira::Frame;
    use kira::backend::mock::{MockBackend, MockBackendSettings};
    use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use uuid::Uuid;

    const SAMPLE_RATE: u32 = 44_100;

    fn audio_output() -> AudioOutput<MockBackend> {
        let settings = AudioManagerSettings::<MockBackend> {
            // The mock backend renders at 1 Hz by default, which is too coarse to play a sound.
            backend_settings: MockBackendSettings {
                sample_rate: SAMPLE_RATE,
            },
            ..default()
        };

        AudioOutput {
            manager: AudioManager::new(settings).ok(),
            instances: HashMap::default(),
            channels: HashMap::default(),
            channel_tracks: HashMap::default(),
            instance_tracks: Vec::new(),
        }
    }

    /// A tenth of a second of audio at full amplitude, so that effects can tell it from silence.
    fn audio_source() -> AudioSource {
        AudioSource {
            sound: StaticSoundData {
                sample_rate: SAMPLE_RATE,
                frames: Arc::from(vec![Frame::from_mono(1.0); SAMPLE_RATE as usize / 10]),
                settings: StaticSoundSettings::default(),
                slice: None,
            },
        }
    }

    /// Render a single buffer, which is what pushes queued sounds and tracks to the renderer and
    /// runs every effect in the graph over their audio.
    fn render(audio_output: &mut AudioOutput<MockBackend>) {
        let backend = audio_output
            .manager
            .as_mut()
            .expect("the mock manager was created")
            .backend_mut();

        backend.on_start_processing();
        backend.process();
    }

    /// Render `duration` worth of audio, cleaning up after every buffer like the plugin does
    /// every frame.
    fn render_for(
        audio_output: &mut AudioOutput<MockBackend>,
        instances: &mut Assets<AudioInstance>,
        duration: Duration,
    ) {
        // The mock backend renders the default internal buffer of 128 frames at a time.
        let buffers = (duration.as_secs_f64() * f64::from(SAMPLE_RATE) / 128.0).ceil() as usize;
        for _ in 0..buffers {
            render(audio_output);
            audio_output.cleanup_stopped_instances(instances);
        }
    }

    /// An effect that keeps producing sound for as long as its handle says so, like a reverb that
    /// never decays.
    struct Ring(Arc<AtomicBool>);

    impl Effect for Ring {
        fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info) {
            if self.0.load(Ordering::Relaxed) {
                input.fill(Frame::from_mono(1.0));
            }
        }
    }

    impl AudioEffect for Ring {
        type Handle = Arc<AtomicBool>;

        fn build_effect(self) -> (Box<dyn Effect>, Self::Handle) {
            let ringing = self.0.clone();

            (Box::new(self), ringing)
        }
    }

    /// An effect recording whether any audio reached it.
    struct Probe(Arc<AtomicBool>);

    impl Probe {
        fn new() -> Self {
            Self(Arc::new(AtomicBool::new(false)))
        }
    }

    impl Effect for Probe {
        fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info) {
            if input.iter().any(|frame| *frame != Frame::ZERO) {
                self.0.store(true, Ordering::Relaxed);
            }
        }
    }

    impl AudioEffect for Probe {
        type Handle = Arc<AtomicBool>;

        fn build_effect(self) -> (Box<dyn Effect>, Self::Handle) {
            let heard = self.0.clone();

            (Box::new(self), heard)
        }
    }

    /// The channel that `AudioChannel<Audio>` plays on.
    fn main_channel() -> Channel {
        Channel::Typed(TypeId::of::<Audio>())
    }

    /// Number of sub-tracks of the given channel's own track.
    fn channel_sub_tracks(audio_output: &AudioOutput<MockBackend>, channel: &Channel) -> usize {
        audio_output
            .channel_tracks
            .get(channel)
            .expect("the channel has a track")
            .num_sub_tracks()
    }

    /// Number of tracks attached directly to the manager.
    fn manager_sub_tracks(audio_output: &AudioOutput<MockBackend>) -> usize {
        audio_output
            .manager
            .as_ref()
            .expect("the mock manager was created")
            .num_sub_tracks()
    }

    /// Play a single sound on `AudioChannel<Audio>`, configured by `configure`.
    fn play_on_main_channel(
        audio_output: &mut AudioOutput<MockBackend>,
        instances: &mut Assets<AudioInstance>,
        configure: impl FnOnce(&mut PlayAudioCommand),
    ) -> Handle<AudioInstance> {
        let mut sources = Assets::<AudioSource>::default();
        let source: Handle<AudioSource> = Handle::Uuid(Uuid::new_v4(), PhantomData);
        let _ = sources.insert(&source, audio_source());

        let channel = AudioChannel::<Audio>::default();
        let instance = {
            let mut command = channel.play(source);
            configure(&mut command);
            command.handle()
        };

        audio_output.play_channel(&sources, &channel, instances);

        instance
    }

    /// Play a sound whose own effect rings for as long as the returned flag is set.
    fn play_ringing_sound(
        audio_output: &mut AudioOutput<MockBackend>,
        instances: &mut Assets<AudioInstance>,
        tail: EffectTail,
    ) -> (Handle<AudioInstance>, Arc<AtomicBool>) {
        let mut ringing = None;
        let instance = play_on_main_channel(audio_output, instances, |command| {
            ringing = Some(command.add_effect(Ring(Arc::new(AtomicBool::new(true)))));
            command.with_effect_tail(tail);
        });

        (instance, ringing.unwrap())
    }

    const TAIL: EffectTail = EffectTail {
        silence: Duration::from_millis(100),
        max: Duration::from_secs(1),
    };

    #[test]
    fn instance_track_is_kept_until_its_effects_fall_silent() {
        let mut audio_output = audio_output();
        let mut instances = Assets::<AudioInstance>::default();
        let (_, ringing) = play_ringing_sound(&mut audio_output, &mut instances, TAIL);

        // The sound itself is over after 100ms, but its effect still rings.
        render_for(
            &mut audio_output,
            &mut instances,
            Duration::from_millis(500),
        );
        assert_eq!(audio_output.instance_tracks.len(), 1);

        ringing.store(false, Ordering::Relaxed);
        render_for(&mut audio_output, &mut instances, Duration::from_millis(50));
        assert_eq!(audio_output.instance_tracks.len(), 1);
        render_for(
            &mut audio_output,
            &mut instances,
            Duration::from_millis(100),
        );
        assert!(audio_output.instance_tracks.is_empty());
    }

    #[test]
    fn instance_track_is_dropped_after_the_max_tail_if_its_effects_keep_ringing() {
        let mut audio_output = audio_output();
        let mut instances = Assets::<AudioInstance>::default();
        let _ringing = play_ringing_sound(&mut audio_output, &mut instances, TAIL);

        render_for(
            &mut audio_output,
            &mut instances,
            Duration::from_millis(900),
        );
        assert_eq!(audio_output.instance_tracks.len(), 1);
        render_for(
            &mut audio_output,
            &mut instances,
            Duration::from_millis(300),
        );
        assert!(audio_output.instance_tracks.is_empty());
    }

    #[test]
    fn stopping_a_sound_drops_its_track_without_a_tail() {
        let mut audio_output = audio_output();
        let mut instances = Assets::<AudioInstance>::default();
        let (instance, _ringing) = play_ringing_sound(&mut audio_output, &mut instances, TAIL);
        render_for(&mut audio_output, &mut instances, Duration::from_millis(20));

        instances
            .get_mut(&instance)
            .unwrap()
            .stop(AudioTween::default());
        render_for(&mut audio_output, &mut instances, Duration::from_millis(30));

        assert!(audio_output.instance_tracks.is_empty());
    }

    #[test]
    fn stopping_a_sound_during_its_tail_fades_its_effects_out_before_dropping_its_track() {
        let mut audio_output = audio_output();
        let mut instances = Assets::<AudioInstance>::default();
        let (instance, _ringing) = play_ringing_sound(&mut audio_output, &mut instances, TAIL);
        render_for(
            &mut audio_output,
            &mut instances,
            Duration::from_millis(200),
        );

        instances
            .get_mut(&instance)
            .unwrap()
            .stop(AudioTween::linear(Duration::from_millis(100)));
        render_for(&mut audio_output, &mut instances, Duration::from_millis(50));
        assert_eq!(audio_output.instance_tracks.len(), 1);
        render_for(&mut audio_output, &mut instances, Duration::from_millis(60));
        assert!(audio_output.instance_tracks.is_empty());
    }

    #[test]
    fn instance_track_is_kept_while_its_sound_plays_even_without_its_instance() {
        let mut audio_output = audio_output();
        let mut instances = Assets::<AudioInstance>::default();
        let (instance, ringing) = play_ringing_sound(&mut audio_output, &mut instances, TAIL);
        ringing.store(false, Ordering::Relaxed);

        instances.remove(&instance);
        render_for(&mut audio_output, &mut instances, Duration::from_millis(80));

        assert_eq!(audio_output.instance_tracks.len(), 1);
    }

    #[test]
    fn instance_effects_run_on_a_sub_track_of_the_channel_track() {
        let mut audio_output = audio_output();
        audio_output.create_channel_track(
            main_channel(),
            AudioTrack::new().with_effect(ReverbBuilder::new()),
        );

        play_on_main_channel(&mut audio_output, &mut Assets::default(), |command| {
            command.with_effect(FilterBuilder::new());
        });

        assert_eq!(audio_output.instances[&main_channel()].len(), 1);
        assert_eq!(audio_output.instance_tracks.len(), 1);
        assert_eq!(channel_sub_tracks(&audio_output, &main_channel()), 1);
        // The channel's own track is the only one hanging off the manager.
        assert_eq!(manager_sub_tracks(&audio_output), 1);
    }

    #[test]
    fn a_sound_with_its_own_effects_is_processed_by_the_channel_effects_as_well() {
        let mut audio_output = audio_output();
        let mut track = AudioTrack::new();
        let channel_probe = track.add_effect(Probe::new());
        audio_output.create_channel_track(main_channel(), track);

        let mut instance_probe = None;
        play_on_main_channel(&mut audio_output, &mut Assets::default(), |command| {
            instance_probe = Some(command.add_effect(Probe::new()));
        });
        render(&mut audio_output);

        assert!(
            instance_probe.unwrap().load(Ordering::Relaxed),
            "the sound did not reach its own effects"
        );
        assert!(
            channel_probe.load(Ordering::Relaxed),
            "the sound bypassed the channel's effects"
        );
    }

    #[test]
    fn instance_effects_run_on_a_manager_track_without_a_channel_track() {
        let mut audio_output = audio_output();

        play_on_main_channel(&mut audio_output, &mut Assets::default(), |command| {
            command.with_effect(FilterBuilder::new());
        });

        assert_eq!(audio_output.instances[&main_channel()].len(), 1);
        assert_eq!(audio_output.instance_tracks.len(), 1);
        assert!(audio_output.channel_tracks.is_empty());
        assert_eq!(manager_sub_tracks(&audio_output), 1);
    }

    #[test]
    fn a_sound_without_effects_plays_on_the_channel_track_itself() {
        let mut audio_output = audio_output();
        audio_output.create_channel_track(
            main_channel(),
            AudioTrack::new().with_effect(ReverbBuilder::new()),
        );

        play_on_main_channel(&mut audio_output, &mut Assets::default(), |_| {});

        assert_eq!(audio_output.instances[&main_channel()].len(), 1);
        assert!(audio_output.instance_tracks.is_empty());
        assert_eq!(channel_sub_tracks(&audio_output, &main_channel()), 0);
        assert_eq!(manager_sub_tracks(&audio_output), 1);
    }

    #[test]
    fn a_sound_plays_without_its_effects_when_its_track_cannot_be_created() {
        let mut audio_output = audio_output();
        audio_output.create_channel_track(
            main_channel(),
            AudioTrack::new().sub_track_capacity(NonZeroUsize::MIN),
        );

        for _ in 0..2 {
            play_on_main_channel(&mut audio_output, &mut Assets::default(), |command| {
                command.with_effect(FilterBuilder::new());
            });
        }

        assert_eq!(audio_output.instances[&main_channel()].len(), 2);
        assert_eq!(audio_output.instance_tracks.len(), 1);
    }

    #[test]
    fn keeps_order_of_commands_to_retry() {
        // we only need this app to conveniently get a assets collection for `AudioSource`...
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), AudioPlugin));
        let audio_source_assets = app
            .world_mut()
            .remove_resource::<Assets<AudioSource>>()
            .unwrap();
        let mut audio_instance_assets = app
            .world_mut()
            .remove_resource::<Assets<AudioInstance>>()
            .unwrap();

        let mut audio_output = audio_output();
        let audio_handle_one: Handle<AudioSource> =
            Handle::<AudioSource>::Uuid(Uuid::new_v4(), PhantomData);
        let audio_handle_two: Handle<AudioSource> =
            Handle::<AudioSource>::Uuid(Uuid::new_v4(), PhantomData);

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
        let audio_source_assets = app
            .world_mut()
            .remove_resource::<Assets<AudioSource>>()
            .unwrap();
        let mut audio_instance_assets = app
            .world_mut()
            .remove_resource::<Assets<AudioInstance>>()
            .unwrap();

        let mut audio_output = audio_output();
        let audio_handle_one: Handle<AudioSource> =
            Handle::<AudioSource>::Uuid(Uuid::new_v4(), PhantomData);
        let audio_handle_two: Handle<AudioSource> =
            Handle::<AudioSource>::Uuid(Uuid::new_v4(), PhantomData);

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
