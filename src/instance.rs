use crate::instance_track::InstanceEffects;
use crate::{AudioTween, PlaybackState};
use bevy_asset::{Asset, Assets, Handle};
use kira::sound::static_sound::StaticSoundHandle;
use kira::{Decibels, Value};

#[derive(Asset, bevy_reflect::TypePath)]
/// Asset for direct audio control
pub struct AudioInstance {
    pub(crate) handle: StaticSoundHandle,
    /// Set if the sound has effects of its own.
    pub(crate) effects: Option<InstanceEffects>,
}

impl AudioInstance {
    /// Pause the audio instance with the given easing
    pub fn pause(&mut self, tween: AudioTween) {
        self.handle.pause(tween.into());
    }

    /// Resume the audio instance with the given easing
    pub fn resume(&mut self, tween: AudioTween) {
        self.resume_with(tween.into());
    }

    pub(crate) fn resume_with(&mut self, tween: kira::Tween) {
        // Kira does not resume a sound that has already stopped, so neither should its effects.
        if let Some(effects) = &mut self.effects
            && self.handle.state() != kira::sound::PlaybackState::Stopped
        {
            effects.resume(tween);
        }
        self.handle.resume(tween);
    }

    /// Stop the audio instance with the given easing
    ///
    /// The sound's own effects fade out with it instead of ringing out.
    pub fn stop(&mut self, tween: AudioTween) {
        self.stop_with(tween.into());
    }

    pub(crate) fn stop_with(&mut self, tween: kira::Tween) {
        if let Some(effects) = &mut self.effects {
            effects.stop(tween);
        }
        self.handle.stop(tween);
    }

    /// Get the state of the audio instance
    pub fn state(&self) -> PlaybackState {
        (&self.handle).into()
    }

    /// Change the volume of the audio instance
    ///
    /// Higher values increase the volume and lower values decrease it.
    /// Setting the volume of a sound to -60dB or lower makes it silent.
    pub fn set_decibels(&mut self, volume: impl Into<Decibels>, tween: AudioTween) {
        self.handle
            .set_volume(Value::Fixed(volume.into()), tween.into());
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
    /// `1.0` is hard right,
    /// `0.0` is center (default),
    /// `-1.0` is hard left.
    pub fn set_panning(&mut self, panning: f32, tween: AudioTween) {
        self.handle
            .set_panning(Value::Fixed(panning.into()), tween.into());
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
