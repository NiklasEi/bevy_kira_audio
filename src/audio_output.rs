//! The internal audio systems and resource

use crate::audio::{AudioCommand, AudioCommandResult, AudioTween, PartialSoundSettings, map_tween};
use std::any::TypeId;

use crate::PlaybackState;
use crate::backend_settings::AudioSettings;
use crate::channel::dynamic::DynamicAudioChannels;
use crate::channel::typed::AudioChannel;
use crate::channel::{Channel, ChannelState};
use crate::effect::{AudioTrack, DEFAULT_EFFECT_TAIL};
use crate::instance::AudioInstance;
use crate::source::AudioSource;
use bevy::asset::{AssetId, Assets, Handle};
use bevy::ecs::change_detection::{NonSendMut, ResMut};
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{NonSend, Res};
use bevy::ecs::world::{FromWorld, World};
use bevy::log::warn;
use bevy::platform::time::Instant;
use kira::ResourceLimitReached;
use kira::backend::{Backend, DefaultBackend};
use kira::track::TrackHandle;
use kira::{AudioManager, Panning};
use kira::{Decibels, PlaybackRate};
use std::collections::HashMap;
use std::time::Duration;

/// The sub-track carrying one sound instance's effects.
///
/// Kira tears a track out of the audio graph as soon as its handle is dropped, without waiting for
/// anything still ringing on it to fade. Dropping the handle the moment playback stops would
/// therefore silence reverb and delay tails instead of letting them ring out, so the handle is held
/// for a while longer.
struct InstanceTrack {
    /// Held only for its `Drop`, which is what removes the sub-track from the audio graph.
    #[expect(dead_code, reason = "kept alive so that dropping it removes the track")]
    handle: TrackHandle,
    /// How long to hold on after the sound stops.
    tail: Duration,
    /// When the tail runs out. `None` while the sound is still playing.
    ///
    /// Effects ring out in wall-clock time, so this deadline is taken against
    /// [`Instant`] rather than any of Bevy's clocks, which can be paused, scaled or missing
    /// entirely when [`TimePlugin`](bevy::time::TimePlugin) is not part of the app.
    expires_at: Option<Instant>,
}

impl InstanceTrack {
    fn new(handle: TrackHandle, tail: Duration) -> Self {
        Self {
            handle,
            tail,
            expires_at: None,
        }
    }

    /// Start the countdown after which the track and its effects are torn down.
    fn start_tail(&mut self, now: Instant) {
        self.expires_at.get_or_insert(now + self.tail);
    }

    /// Report whether the track should be kept at `now`.
    fn keep_alive(&self, now: Instant) -> bool {
        self.expires_at.is_none_or(|expires_at| now < expires_at)
    }
}

/// Non-send resource that acts as audio output
///
/// This struct holds the [`AudioManager`] to play audio through. It also
/// keeps track of all audio instance handles and which sounds are playing in which channel.
pub(crate) struct AudioOutput<B: Backend = DefaultBackend> {
    manager: Option<AudioManager<B>>,
    instances: HashMap<Channel, Vec<Handle<AudioInstance>>>,
    channels: HashMap<Channel, ChannelState>,
    channel_tracks: HashMap<Channel, TrackHandle>,
    instance_tracks: HashMap<AssetId<AudioInstance>, InstanceTrack>,
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
            instance_tracks: HashMap::default(),
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
                sound.settings.playback_rate = kira::Value::Fixed(PlaybackRate(0.0));
            }
        }
        if partial_sound_settings.paused {
            sound.settings.playback_rate = kira::Value::Fixed(PlaybackRate(0.0));
        }
        partial_sound_settings.apply(&mut sound);

        // Determine where to play the sound based on per-instance and channel tracks
        let instance_track = partial_sound_settings.track.lock().take();

        let sound_handle = if let Some(track) = instance_track {
            // Per-instance effects: create a sub-track for this instance
            match self.add_instance_track(channel, *track) {
                Ok(mut track_handle) => {
                    let result = track_handle.play(sound);
                    if result.is_ok() {
                        let tail = partial_sound_settings
                            .effect_tail
                            .unwrap_or(DEFAULT_EFFECT_TAIL);
                        self.instance_tracks
                            .insert(instance_handle.id(), InstanceTrack::new(track_handle, tail));
                    }
                    result
                }
                Err(error) => {
                    warn!("Failed to create sub-track: {:?}", error);
                    return AudioCommandResult::Ok;
                }
            }
        } else if let Some(track_handle) = self.channel_tracks.get_mut(channel) {
            // Channel-level effects: play on the channel's sub-track
            track_handle.play(sound)
        } else {
            // No effects: play on the main track
            self.manager.as_mut().unwrap().play(sound)
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

    /// Create the sub-track carrying one sound's own effects.
    ///
    /// The track is nested under the channel's track when the channel has one, so that the
    /// channel's effects still run after this sound's. Kira processes a track's children before
    /// the track's own effects, which makes the resulting chain sound → instance effects →
    /// channel effects → main track. Channels without a track of their own attach it directly to
    /// the main track instead.
    fn add_instance_track(
        &mut self,
        channel: &Channel,
        track: AudioTrack,
    ) -> Result<TrackHandle, ResourceLimitReached> {
        let track = track.into_inner();

        if let Some(channel_track) = self.channel_tracks.get_mut(channel) {
            channel_track.add_sub_track(track)
        } else {
            self.manager.as_mut().unwrap().add_sub_track(track)
        }
    }

    pub(crate) fn create_channel_track(&mut self, channel: Channel, track: AudioTrack) {
        if let Some(manager) = self.manager.as_mut() {
            match manager.add_sub_track(track.into_inner()) {
                Ok(track_handle) => {
                    if self.channel_tracks.insert(channel, track_handle).is_some() {
                        warn!(
                            "An audio track was already registered for this channel and has been \
                             replaced. Effect handles for the previous track will no longer control \
                             this channel. Ensure `add_audio_channel_with_track` is called only once \
                             per channel type."
                        );
                    }
                }
                Err(error) => {
                    warn!("Failed to create channel sub-track: {:?}", error);
                }
            }
        }
    }

    pub(crate) fn cleanup_stopped_instances(&mut self, instances: &mut Assets<AudioInstance>) {
        let now = Instant::now();

        for handles in self.instances.values_mut() {
            handles.retain(|handle| {
                let stopped = instances.get(handle).is_none_or(|instance| {
                    instance.handle.state() == kira::sound::PlaybackState::Stopped
                });
                if stopped {
                    // Let the effects on this instance's track ring out before tearing it down.
                    if let Some(track) = self.instance_tracks.get_mut(&handle.id()) {
                        track.start_tail(now);
                    }
                }

                !stopped
            });
        }

        // Dropping the handle is what removes the sub-track from kira's audio graph.
        self.instance_tracks
            .retain(|_, track| track.keep_alive(now));
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
    use crate::effect::{AudioEffect, Effect, FilterBuilder, Info, ReverbBuilder};
    use crate::{Audio, AudioPlugin, PlayAudioCommand};
    use bevy::asset::AssetPlugin;
    use bevy::prelude::*;
    use kira::AudioManagerSettings;
    use kira::Frame;
    use kira::backend::mock::{MockBackend, MockBackendSettings};
    use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
    use kira::track::TrackBuilder;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use uuid::Uuid;

    fn instance_track(tail: Duration) -> InstanceTrack {
        let mut manager =
            AudioManager::new(AudioManagerSettings::<MockBackend>::default()).unwrap();

        InstanceTrack::new(manager.add_sub_track(TrackBuilder::new()).unwrap(), tail)
    }

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
            instance_tracks: HashMap::default(),
        }
    }

    /// A second of audio at full amplitude, so that effects can tell it from silence.
    fn audio_source() -> AudioSource {
        AudioSource {
            sound: StaticSoundData {
                sample_rate: SAMPLE_RATE,
                frames: Arc::from(vec![Frame::from_mono(1.0); SAMPLE_RATE as usize]),
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
        configure: impl FnOnce(&mut PlayAudioCommand),
    ) {
        let mut sources = Assets::<AudioSource>::default();
        let mut instances = Assets::<AudioInstance>::default();
        let source: Handle<AudioSource> = Handle::Uuid(Uuid::new_v4(), PhantomData);
        let _ = sources.insert(&source, audio_source());

        let channel = AudioChannel::<Audio>::default();
        configure(&mut channel.play(source));

        audio_output.play_channel(&sources, &channel, &mut instances);
    }

    #[test]
    fn instance_track_is_kept_while_its_sound_plays() {
        let track = instance_track(Duration::from_millis(100));
        let now = Instant::now();

        assert!(track.keep_alive(now + Duration::from_secs(10)));
    }

    #[test]
    fn instance_track_outlives_its_sound_by_the_effect_tail() {
        let mut track = instance_track(Duration::from_millis(100));
        let now = Instant::now();
        track.start_tail(now);

        assert!(track.keep_alive(now + Duration::from_millis(60)));
        assert!(track.keep_alive(now + Duration::from_millis(99)));
        assert!(!track.keep_alive(now + Duration::from_millis(100)));
    }

    #[test]
    fn a_zero_effect_tail_drops_the_instance_track_right_away() {
        let mut track = instance_track(Duration::ZERO);
        let now = Instant::now();
        track.start_tail(now);

        assert!(!track.keep_alive(now));
    }

    #[test]
    fn instance_effects_run_on_a_sub_track_of_the_channel_track() {
        let mut audio_output = audio_output();
        audio_output.create_channel_track(
            main_channel(),
            AudioTrack::new().with_effect(ReverbBuilder::new()),
        );

        play_on_main_channel(&mut audio_output, |command| {
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
        play_on_main_channel(&mut audio_output, |command| {
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

        play_on_main_channel(&mut audio_output, |command| {
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

        play_on_main_channel(&mut audio_output, |_| {});

        assert_eq!(audio_output.instances[&main_channel()].len(), 1);
        assert!(audio_output.instance_tracks.is_empty());
        assert_eq!(channel_sub_tracks(&audio_output, &main_channel()), 0);
        assert_eq!(manager_sub_tracks(&audio_output), 1);
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
