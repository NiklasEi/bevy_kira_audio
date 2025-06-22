use crate::audio::{
    AudioCommand, FadeIn, FadeOut, PlayAudioCommand, PlayAudioSettings, TweenCommand,
    TweenCommandKind,
};
use crate::channel::AudioCommandQue;
use crate::instance::AudioInstance;
use crate::{AudioControl, AudioSource, PlaybackState};
use bevy::asset::{AssetId, Handle};
use bevy::ecs::resource::Resource;
use bevy::platform::collections::HashMap;
use kira::{Decibels, Panning};
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::marker::PhantomData;

/// Channel to play and control audio
///
/// Add your own channels via [`add_audio_channel`](AudioApp::add_audio_channel).
/// By default, there is only the [`AudioChannel<MainTrack>`](crate::Audio) channel.
#[derive(Resource)]
pub struct AudioChannel<T> {
    pub(crate) commands: RwLock<VecDeque<AudioCommand>>,
    pub(crate) states: HashMap<AssetId<AudioInstance>, PlaybackState>,
    _marker: PhantomData<T>,
}

impl<T> Default for AudioChannel<T> {
    fn default() -> Self {
        AudioChannel::<T> {
            commands: Default::default(),
            states: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<T> AudioCommandQue for AudioChannel<T> {
    fn que(&self, command: AudioCommand) {
        self.commands.write().push_front(command)
    }
}

impl<T> AudioControl for AudioChannel<T> {
    /// Play audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     audio.play(asset_server.load("audio.mp3"));
    /// }
    /// ```
    fn play(&self, audio_source: Handle<AudioSource>) -> PlayAudioCommand {
        PlayAudioCommand::new(audio_source, self)
    }

    /// Stop all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.stop();
    /// }
    /// ```
    fn stop(&self) -> TweenCommand<FadeOut> {
        TweenCommand::new(TweenCommandKind::Stop, self)
    }

    /// Pause all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.pause();
    /// }
    /// ```
    fn pause(&self) -> TweenCommand<FadeOut> {
        TweenCommand::new(TweenCommandKind::Pause, self)
    }

    /// Resume all audio
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.resume();
    /// }
    /// ```
    fn resume(&self) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::Resume, self)
    }

    /// Set the volume
    ///
    /// The default value is 1.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_volume(0.5);
    /// }
    /// ```
    fn set_volume(&self, volume: impl Into<Decibels>) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::SetVolume(volume.into()), self)
    }

    /// Set panning
    ///
    /// The default value is 0.5
    /// Values up to 1 pan to the right
    /// Values down to 0 pan to the left
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_panning(kira::Panning(0.9));
    /// }
    /// ```
    fn set_panning(&self, panning: Panning) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::SetPanning(panning), self)
    }

    /// Set playback rate
    ///
    /// The default value is 1
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::prelude::*;
    ///
    /// fn my_system(audio: Res<Audio>) {
    ///     audio.set_playback_rate(2.0);
    /// }
    /// ```
    fn set_playback_rate(&self, playback_rate: f64) -> TweenCommand<FadeIn> {
        TweenCommand::new(TweenCommandKind::SetPlaybackRate(playback_rate), self)
    }

    /// Get state for a playback instance.
    fn state(&self, instance_handle: &Handle<AudioInstance>) -> PlaybackState {
        self.states
            .get(&instance_handle.id())
            .cloned()
            .unwrap_or_else(|| {
                self.commands
                    .read()
                    .iter()
                    .find(|command| match command {
                        AudioCommand::Play(PlayAudioSettings {
                            instance_handle: handle,
                            settings: _,
                            source: _,
                        }) => handle.id() == instance_handle.id(),
                        _ => false,
                    })
                    .map(|_| PlaybackState::Queued)
                    .unwrap_or(PlaybackState::Stopped)
            })
    }

    /// Returns `true` if there is any sound in this channel that is in the state `Playing`, `Pausing`, or `Stopping`
    ///
    /// If there are only `Stopped`, `Paused`, or `Queued` sounds, the method will return `false`.
    /// The same result is returned if there are no sounds in the channel at all.
    fn is_playing_sound(&self) -> bool {
        self.states
            .iter()
            .fold(false, |playing, (_, state)| match state {
                PlaybackState::Playing { .. }
                | PlaybackState::Pausing { .. }
                | PlaybackState::Stopping { .. } => true,
                _ => playing,
            })
    }
}

#[cfg(test)]
mod test {
    use crate::channel::typed::AudioChannel;
    use crate::channel::*;
    use crate::Audio;
    use bevy::asset::{AssetId, Handle};
    use uuid::Uuid;

    #[test]
    fn state_is_queued_if_command_is_queued() {
        let audio = AudioChannel::<Audio>::default();
        let audio_handle: Handle<AudioSource> =
            Handle::<AudioSource>::Weak(AssetId::<AudioSource>::default());
        let instance_handle = audio.play(audio_handle).handle();

        assert_eq!(audio.state(&instance_handle), PlaybackState::Queued);
    }

    #[test]
    fn state_is_stopped_if_command_is_not_queued_and_id_not_in_state_map() {
        let audio = AudioChannel::<Audio>::default();
        let instance_handle = Handle::<AudioInstance>::Weak(AssetId::<AudioInstance>::default());

        assert_eq!(audio.state(&instance_handle), PlaybackState::Stopped);
    }

    #[test]
    fn state_is_fetched_from_state_map() {
        let mut audio = AudioChannel::<Audio>::default();
        let instance_handle = Handle::<AudioInstance>::Weak(AssetId::<AudioInstance>::default());
        audio.states.insert(
            instance_handle.id(),
            PlaybackState::Pausing { position: 42. },
        );

        assert_eq!(
            audio.state(&instance_handle),
            PlaybackState::Pausing { position: 42. }
        );
    }

    #[test]
    fn finds_playing_sound() {
        let mut audio = AudioChannel::<Audio>::default();
        audio.states.insert(
            Uuid::from_u128(43290473942075938).into(),
            PlaybackState::Queued,
        );
        audio.states.insert(
            Uuid::from_u128(432952340473942075938).into(),
            PlaybackState::Paused { position: 42. },
        );
        audio.states.insert(
            Uuid::from_u128(46254624324354345324).into(),
            PlaybackState::Stopped,
        );
        assert!(!audio.is_playing_sound());

        audio.states.insert(
            Uuid::from_u128(70973842759324739).into(),
            PlaybackState::Playing { position: 42. },
        );
        assert!(audio.is_playing_sound());
    }
}
