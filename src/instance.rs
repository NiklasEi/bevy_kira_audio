use crate::{AudioTween, PlaybackState};
use bevy::asset::{Asset, Assets, Handle};
use kira::sound::static_sound::StaticSoundHandle;
use kira::tween::Value;
use kira::{CommandError, Volume};
use thiserror::Error;

#[derive(Asset, bevy::reflect::TypePath)]
/// Asset for direct audio control
pub struct AudioInstance {
    pub(crate) handle: StaticSoundHandle,
}

/// Errors that can occur when directly controlling audio
#[derive(Error, Debug)]
pub enum AudioCommandError {
    /// The audio command que of the audio manager is full
    #[error("the audio thread could not handle the command, because its command que is full")]
    CommandQueueFull,

    /// Something went wrong when handling the command in the audio thread
    #[error("an error occurred while handling the command in the audio thread")]
    AudioThreadError,
}

impl From<CommandError> for AudioCommandError {
    fn from(kira_error: CommandError) -> Self {
        match kira_error {
            CommandError::CommandQueueFull => AudioCommandError::CommandQueueFull,
            _ => AudioCommandError::AudioThreadError,
        }
    }
}

impl AudioInstance {
    /// Pause the audio instance with the given easing
    pub fn pause(&mut self, tween: AudioTween) -> Option<AudioCommandError> {
        self.handle
            .pause(tween.into())
            .err()
            .map(|kira_error| kira_error.into())
    }

    /// Resume the audio instance with the given easing
    pub fn resume(&mut self, tween: AudioTween) -> Option<AudioCommandError> {
        self.handle
            .resume(tween.into())
            .err()
            .map(|kira_error| kira_error.into())
    }

    /// Stop the audio instance with the given easing
    pub fn stop(&mut self, tween: AudioTween) -> Option<AudioCommandError> {
        self.handle
            .stop(tween.into())
            .err()
            .map(|kira_error| kira_error.into())
    }

    /// Get the state of the audio instance
    pub fn state(&self) -> PlaybackState {
        (&self.handle).into()
    }

    /// Set the volume of the audio instance
    ///
    /// Default is `1.0`
    pub fn set_volume(
        &mut self,
        volume: impl Into<Value<Volume>>,
        tween: AudioTween,
    ) -> Option<AudioCommandError> {
        self.handle
            .set_volume(volume, tween.into())
            .err()
            .map(|kira_error| kira_error.into())
    }

    /// Sets the playback rate of the sound.
    ///
    /// Changing the playback rate will change both the speed
    /// and pitch of the sound.
    pub fn set_playback_rate(
        &mut self,
        playback_rate: f64,
        tween: AudioTween,
    ) -> Option<AudioCommandError> {
        self.handle
            .set_playback_rate(playback_rate, tween.into())
            .err()
            .map(|kira_error| kira_error.into())
    }

    /// Sets the panning of the sound
    ///
    /// `0.0` is hard left,
    /// `0.5` is center (default)
    /// `1.0` is hard right.
    pub fn set_panning(&mut self, panning: f64, tween: AudioTween) -> Option<AudioCommandError> {
        self.handle
            .set_panning(panning, tween.into())
            .err()
            .map(|kira_error| kira_error.into())
    }

    /// Sets the playback position to the specified time in seconds.
    pub fn seek_to(&mut self, position: f64) -> Option<AudioCommandError> {
        self.handle
            .seek_to(position)
            .err()
            .map(|kira_error| kira_error.into())
    }

    /// Moves the playback position by the specified amount of time in seconds.
    pub fn seek_by(&mut self, amount: f64) -> Option<AudioCommandError> {
        self.handle
            .seek_by(amount)
            .err()
            .map(|kira_error| kira_error.into())
    }
}

/// Extension trait to remove some boilerplate when
pub trait AudioInstanceAssetsExt {
    /// Get the playback state of the audio instance
    ///
    /// # Note
    /// A return value of [`PlaybackState::Stopped`] might be either a stopped instance or a
    /// queued one! To be able to differentiate the two, you need to query the state on the
    /// channel that the sound was played on.
    fn state(&self, instance_handle: &Handle<AudioInstance>) -> PlaybackState;
}

impl AudioInstanceAssetsExt for Assets<AudioInstance> {
    fn state(&self, instance_handle: &Handle<AudioInstance>) -> PlaybackState {
        self.get(instance_handle)
            .map(|instance| instance.state())
            .unwrap_or(PlaybackState::Stopped)
    }
}
