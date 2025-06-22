use crate::{AudioTween, PlaybackState};
use bevy::asset::{Asset, Assets, Handle};
use kira::{sound::static_sound::StaticSoundHandle, Decibels, Panning, Value};

#[derive(Asset, bevy::reflect::TypePath)]
/// Asset for direct audio control
pub struct AudioInstance {
    pub(crate) handle: StaticSoundHandle,
}

impl AudioInstance {
    /// Pause the audio instance with the given easing
    pub fn pause(&mut self, tween: AudioTween) {
        self.handle.pause(tween.into());
    }

    /// Resume the audio instance with the given easing
    pub fn resume(&mut self, tween: AudioTween) {
        self.handle.resume(tween.into());
    }

    /// Stop the audio instance with the given easing
    pub fn stop(&mut self, tween: AudioTween) {
        self.handle.stop(tween.into());
    }

    /// Get the state of the audio instance
    pub fn state(&self) -> PlaybackState {
        (&self.handle).into()
    }

    /// Set the volume of the audio instance
    ///
    /// Default is `1.0`
    pub fn set_volume(&mut self, volume: impl Into<Value<Decibels>>, tween: AudioTween) {
        self.handle.set_volume(volume, tween.into());
    }

    /// Sets the playback rate of the sound.
    ///
    /// Changing the playback rate will change both the speed
    /// and pitch of the sound.
    pub fn set_playback_rate(&mut self, playback_rate: f64, tween: AudioTween) {
        self.handle.set_playback_rate(playback_rate, tween.into());
    }

    /// Sets the panning of the sound
    ///
    /// `0.0` is hard left,
    /// `0.5` is center (default)
    /// `1.0` is hard right.
    pub fn set_panning(&mut self, panning: Panning, tween: AudioTween) {
        self.handle.set_panning(panning, tween.into());
    }

    /// Sets the playback position to the specified time in seconds.
    pub fn seek_to(&mut self, position: f64) {
        self.handle.seek_to(position);
    }

    /// Moves the playback position by the specified amount of time in seconds.
    pub fn seek_by(&mut self, amount: f64) {
        self.handle.seek_by(amount);
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
