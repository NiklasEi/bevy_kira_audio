//! The track carrying a single sound's own effects

use crate::effect::{AudioTrack, EffectTail};
use kira::effect::Effect;
use kira::effect::volume_control::{VolumeControlBuilder, VolumeControlHandle};
use kira::info::Info;
use kira::track::{TrackBuilder, TrackHandle};
use kira::{Decibels, Frame, ResourceLimitReached, Tween};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Output below this amplitude (-60 dB) counts as silence.
const SILENCE_THRESHOLD: f32 = 0.001;

/// How long effects still ringing at [`EffectTail::max`] take to fade out, the same as
/// [`AudioTween::default`](crate::AudioTween::default).
const MAX_TAIL_FADE_OUT: Duration = Duration::from_millis(10);

/// The track of one sound with effects of its own.
///
/// Kira removes a track as soon as its handle is dropped, cutting off whatever is still ringing on
/// it. So the handle is kept until the sound is gone and its effects have rung out, or have faded
/// out after the sound was stopped.
pub(crate) struct InstanceTrack {
    pub(crate) handle: TrackHandle,
    tail: Arc<TailState>,
}

impl InstanceTrack {
    /// Build the sound's track and add it with `add_sub_track`.
    ///
    /// Also returns the controls for the sound's [`AudioInstance`](crate::AudioInstance).
    pub(crate) fn add(
        track: AudioTrack,
        tail: EffectTail,
        add_sub_track: impl FnOnce(TrackBuilder) -> Result<TrackHandle, ResourceLimitReached>,
    ) -> Result<(Self, InstanceEffects), ResourceLimitReached> {
        let mut builder = track.into_inner();
        let volume = builder.add_effect(VolumeControlBuilder::new(Decibels::IDENTITY));
        let tail_state = Arc::new(TailState::default());
        builder.add_built_effect(Box::new(TailMonitor::new(tail, tail_state.clone())));

        let track = Self {
            handle: add_sub_track(builder)?,
            tail: tail_state.clone(),
        };

        Ok((
            track,
            InstanceEffects {
                volume,
                tail: tail_state,
            },
        ))
    }

    /// Whether the track has to be kept for the sound or its effects.
    pub(crate) fn keep_alive(&self) -> bool {
        if self.handle.num_sounds() > 0 {
            return true;
        }
        self.tail.sound_finished.store(true, Ordering::Relaxed);

        !self.tail.rung_out.load(Ordering::Relaxed)
    }
}

/// Lets an [`AudioInstance`](crate::AudioInstance) cut off the effects of its sound when stopped.
pub(crate) struct InstanceEffects {
    volume: VolumeControlHandle,
    tail: Arc<TailState>,
}

impl InstanceEffects {
    pub(crate) fn stop(&mut self, tween: Tween) {
        self.tail.stopped.store(true, Ordering::Relaxed);
        self.volume.set_volume(Decibels::SILENCE, tween);
    }

    pub(crate) fn resume(&mut self, tween: Tween) {
        self.tail.stopped.store(false, Ordering::Relaxed);
        self.volume.set_volume(Decibels::IDENTITY, tween);
    }
}

#[derive(Default)]
struct TailState {
    /// Set once the sound is gone, which starts the tail.
    sound_finished: AtomicBool,
    /// Set while the sound is stopped, which ends the tail as soon as its effects have faded out.
    stopped: AtomicBool,
    /// Set by the monitor once the tail is over.
    rung_out: AtomicBool,
}

/// The last effect on a sound's track, deciding when the tail is over.
///
/// Running on the audio thread, it measures the tail in audio that was actually played.
struct TailMonitor {
    tail: EffectTail,
    state: Arc<TailState>,
    silent_for: f64,
    finished_for: f64,
    /// Fades the output out once the tail has reached its maximum.
    gain: f32,
}

impl TailMonitor {
    fn new(tail: EffectTail, state: Arc<TailState>) -> Self {
        Self {
            tail,
            state,
            silent_for: 0.0,
            finished_for: 0.0,
            gain: 1.0,
        }
    }

    fn fade_out(&mut self, input: &mut [Frame], dt: f64) {
        let step = (dt / MAX_TAIL_FADE_OUT.as_secs_f64()) as f32;
        for frame in input {
            self.gain = (self.gain - step).max(0.0);
            *frame *= self.gain;
        }
    }
}

impl Effect for TailMonitor {
    fn process(&mut self, input: &mut [Frame], dt: f64, _info: &Info) {
        let elapsed = dt * input.len() as f64;
        let finished = self.state.sound_finished.load(Ordering::Relaxed);
        if finished {
            self.finished_for += elapsed;
            if self.finished_for >= self.tail.max.as_secs_f64() {
                self.fade_out(input, dt);
            }
        }

        let silent = input.iter().all(|frame| {
            frame.left.abs() < SILENCE_THRESHOLD && frame.right.abs() < SILENCE_THRESHOLD
        });
        self.silent_for = if silent {
            self.silent_for + elapsed
        } else {
            0.0
        };

        let stopped = self.state.stopped.load(Ordering::Relaxed);
        if finished
            && ((stopped && silent)
                || self.silent_for >= self.tail.silence.as_secs_f64()
                || self.gain == 0.0)
        {
            self.state.rung_out.store(true, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use kira::info::MockInfoBuilder;

    const SAMPLE_RATE: f64 = 44_100.0;

    /// Run one buffer of 128 loud frames through the monitor and return its last frame.
    fn process_loud_buffer(monitor: &mut TailMonitor) -> Frame {
        let mut buffer = [Frame::from_mono(1.0); 128];
        monitor.process(
            &mut buffer,
            1.0 / SAMPLE_RATE,
            &MockInfoBuilder::new().build(),
        );

        buffer[127]
    }

    #[test]
    fn effects_still_ringing_at_the_max_tail_fade_out_before_the_track_goes() {
        let state = Arc::new(TailState::default());
        let tail = EffectTail {
            silence: Duration::from_secs(1),
            max: Duration::ZERO,
        };
        let mut monitor = TailMonitor::new(tail, state.clone());
        state.sound_finished.store(true, Ordering::Relaxed);

        let fading = process_loud_buffer(&mut monitor);
        assert!(fading.left > 0.0 && fading.left < 1.0);
        assert!(!state.rung_out.load(Ordering::Relaxed));

        for _ in 0..3 {
            process_loud_buffer(&mut monitor);
        }
        assert_eq!(process_loud_buffer(&mut monitor), Frame::ZERO);
        assert!(state.rung_out.load(Ordering::Relaxed));
    }
}
