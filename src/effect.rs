//! Audio effects
//!
//! Effects modify the audio signal of a single sound instance or of a whole channel.
//!
//! Add an effect to one sound with [`PlayAudioCommand::add_effect`](crate::PlayAudioCommand::add_effect),
//! or to an entire channel by building an [`AudioTrack`] and passing it to
//! [`add_audio_channel_with_track`](crate::AudioApp::add_audio_channel_with_track).
//!
//! Every effect is configured through a builder. Adding it returns a handle that controls the
//! effect while it plays. All handle setters take an [`AudioTween`] describing how the change is
//! interpolated, just like the rest of this crate's API.
//!
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_kira_audio::prelude::*;
//!
//! fn play(audio: Res<Audio>, asset_server: Res<AssetServer>) {
//!     audio
//!         .play(asset_server.load("sounds/loop.ogg"))
//!         .with_effect(FilterBuilder::new().mode(FilterMode::LowPass).cutoff(300.0))
//!         .looped();
//! }
//! ```
//!
//! Effects added to a single sound run on a track of that sound's own. Reverb and delay keep
//! sounding after their input has gone quiet, so that track outlives the sound by
//! [`DEFAULT_EFFECT_TAIL`]; see
//! [`with_effect_tail`](crate::PlayAudioCommand::with_effect_tail) to change how long.
//!
//! # Custom effects
//!
//! Implement [`Effect`] to process audio frames yourself, then implement [`AudioEffect`] for a
//! builder that creates it. Your builder can then be used anywhere the built-in ones are.
//!
//! ```
//! use bevy_kira_audio::prelude::*;
//!
//! /// Silences the right channel.
//! struct LeftOnly;
//!
//! impl Effect for LeftOnly {
//!     fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info) {
//!         for frame in input {
//!             frame.right = 0.0;
//!         }
//!     }
//! }
//!
//! struct LeftOnlyBuilder;
//!
//! impl AudioEffect for LeftOnlyBuilder {
//!     type Handle = ();
//!
//!     fn build_effect(self) -> (Box<dyn Effect>, Self::Handle) {
//!         (Box::new(LeftOnly), ())
//!     }
//! }
//! ```

use crate::audio::AudioTween;
use kira::effect::EffectBuilder as KiraEffectBuilder;
use kira::track::TrackBuilder;
use std::time::Duration;

pub use kira::effect::Effect;
pub use kira::info::Info;
pub use kira::{Decibels, Frame, Mix, Panning};

pub use kira::effect::compressor::CompressorBuilder;
pub use kira::effect::distortion::{DistortionBuilder, DistortionKind};
pub use kira::effect::eq_filter::{EqFilterBuilder, EqFilterKind};
pub use kira::effect::filter::{FilterBuilder, FilterMode};
pub use kira::effect::panning_control::PanningControlBuilder;
pub use kira::effect::reverb::ReverbBuilder;
pub use kira::effect::volume_control::VolumeControlBuilder;

/// How long a sound's effects keep running after the sound itself has stopped.
///
/// Effects like reverb and delay go on producing sound after their input has gone quiet. A sound
/// with per-instance effects plays on its own track, and tearing that track down the moment the
/// sound ends would cut those tails off mid-ring. The track is therefore kept for this long
/// afterwards. Override it per sound with
/// [`with_effect_tail`](crate::PlayAudioCommand::with_effect_tail).
pub const DEFAULT_EFFECT_TAIL: Duration = Duration::from_secs(2);

/// Something that can be added to an audio track as an effect.
///
/// This is implemented for every built-in effect builder. Implement it for your own type to use a
/// custom [`Effect`] with this plugin (see the [module documentation](self#custom-effects)).
pub trait AudioEffect {
    /// Handle used to control the effect while it is running.
    ///
    /// Use `()` if the effect has nothing to control at runtime.
    type Handle;

    /// Build the effect together with a handle to control it.
    fn build_effect(self) -> (Box<dyn Effect>, Self::Handle);
}

/// A track that sounds are played on, carrying a chain of effects.
///
/// Pass it to [`add_audio_channel_with_track`](crate::AudioApp::add_audio_channel_with_track) to
/// apply its effects to every sound played on that channel.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_kira_audio::prelude::*;
///
/// #[derive(Resource)]
/// struct MusicChannel;
///
/// fn main() {
///     let mut track = AudioTrack::new();
///     let _reverb = track.add_effect(ReverbBuilder::new());
///
///     App::new()
///         .add_plugins((DefaultPlugins, AudioPlugin))
///         .add_audio_channel_with_track::<MusicChannel>(track)
///         .run();
/// }
/// ```
pub struct AudioTrack(TrackBuilder);

impl AudioTrack {
    /// Create a new track with no effects.
    #[must_use]
    pub fn new() -> Self {
        Self(TrackBuilder::new())
    }

    /// Create the track carrying a single sound instance's own effects.
    ///
    /// Such a track hosts exactly the one sound: nesting anything under an instance
    /// track fails with `ResourceLimitReached`.
    pub(crate) fn for_instance() -> Self {
        Self(TrackBuilder::new().sound_capacity(1).sub_track_capacity(0))
    }

    /// Add an effect to this track and return its handle for runtime control.
    pub fn add_effect<E: AudioEffect>(&mut self, effect: E) -> E::Handle {
        let (built, handle) = effect.build_effect();
        self.0.add_built_effect(built);

        handle
    }

    /// Add an effect to this track, discarding its handle.
    ///
    /// Use [`add_effect`](Self::add_effect) if you need to control the effect at runtime.
    #[must_use = "This method consumes self and returns a modified AudioTrack, so the return value should be used"]
    pub fn with_effect<E: AudioEffect>(mut self, effect: E) -> Self {
        self.add_effect(effect);

        self
    }

    /// Set the volume of this track in Decibels.
    #[must_use = "This method consumes self and returns a modified AudioTrack, so the return value should be used"]
    pub fn volume(self, volume: impl Into<Decibels>) -> Self {
        let volume: Decibels = volume.into();

        Self(self.0.volume(volume))
    }

    /// Set the maximum number of sounds that can play on this track at a time.
    #[must_use = "This method consumes self and returns a modified AudioTrack, so the return value should be used"]
    pub fn sound_capacity(self, capacity: usize) -> Self {
        Self(self.0.sound_capacity(capacity))
    }

    /// Set the maximum number of sub-tracks this track can hold.
    ///
    /// Every sound played on this channel with per-instance effects (see
    /// [`add_effect`](crate::PlayAudioCommand::add_effect)) runs on a sub-track of this one and
    /// takes a slot for as long as it plays, plus its
    /// [effect tail](crate::PlayAudioCommand::with_effect_tail).
    #[must_use = "This method consumes self and returns a modified AudioTrack, so the return value should be used"]
    pub fn sub_track_capacity(self, capacity: usize) -> Self {
        Self(self.0.sub_track_capacity(capacity))
    }

    /// Keep the track alive until all sounds on it have finished playing.
    #[must_use = "This method consumes self and returns a modified AudioTrack, so the return value should be used"]
    pub fn persist_until_sounds_finish(self, persist: bool) -> Self {
        Self(self.0.persist_until_sounds_finish(persist))
    }

    pub(crate) fn into_inner(self) -> TrackBuilder {
        self.0
    }
}

impl Default for AudioTrack {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapter handing an already built effect to Kira's builder APIs.
struct PreBuilt(Box<dyn Effect>);

impl KiraEffectBuilder for PreBuilt {
    type Handle = ();

    fn build(self) -> (Box<dyn Effect>, Self::Handle) {
        (self.0, ())
    }
}

/// Configures a delay effect, adding echoes to a sound.
///
/// Effects can be added to the feedback loop, so that every echo is processed by them.
pub struct DelayBuilder(kira::effect::delay::DelayBuilder);

impl DelayBuilder {
    /// Create a new `DelayBuilder` with the default settings.
    #[must_use]
    pub fn new() -> Self {
        Self(kira::effect::delay::DelayBuilder::new())
    }

    /// Set the amount of time the input audio is delayed by.
    #[must_use = "This method consumes self and returns a modified DelayBuilder, so the return value should be used"]
    pub fn delay_time(self, delay_time: Duration) -> Self {
        Self(self.0.delay_time(delay_time))
    }

    /// Set the amount of feedback in Decibels.
    #[must_use = "This method consumes self and returns a modified DelayBuilder, so the return value should be used"]
    pub fn feedback(self, feedback: impl Into<Decibels>) -> Self {
        let feedback: Decibels = feedback.into();

        Self(self.0.feedback(feedback))
    }

    /// Set how much dry (unprocessed) signal is blended with the wet (processed) signal.
    #[must_use = "This method consumes self and returns a modified DelayBuilder, so the return value should be used"]
    pub fn mix(self, mix: impl Into<Mix>) -> Self {
        let mix: Mix = mix.into();

        Self(self.0.mix(mix))
    }

    /// Add an effect to the feedback loop and return its handle for runtime control.
    pub fn add_feedback_effect<E: AudioEffect>(&mut self, effect: E) -> E::Handle {
        let (built, handle) = effect.build_effect();
        self.0.add_feedback_effect(PreBuilt(built));

        handle
    }

    /// Add an effect to the feedback loop, discarding its handle.
    ///
    /// Use [`add_feedback_effect`](Self::add_feedback_effect) if you need to control the effect at
    /// runtime.
    #[must_use = "This method consumes self and returns a modified DelayBuilder, so the return value should be used"]
    pub fn with_feedback_effect<E: AudioEffect>(mut self, effect: E) -> Self {
        self.add_feedback_effect(effect);

        self
    }
}

impl Default for DelayBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffect for DelayBuilder {
    type Handle = DelayHandle;

    fn build_effect(self) -> (Box<dyn Effect>, Self::Handle) {
        let (effect, handle) = KiraEffectBuilder::build(self.0);

        (effect, DelayHandle(handle))
    }
}

/// Define a handle wrapping a Kira effect handle, forwarding its tweened setters.
macro_rules! effect_handle {
    (
        $(#[$handle_doc:meta])*
        $name:ident($inner:ty) {
            $(
                $(#[$setter_doc:meta])*
                $setter:ident($param:ident: $public:ty => $concrete:ty);
            )*
        }
    ) => {
        $(#[$handle_doc])*
        #[derive(Debug)]
        pub struct $name($inner);

        impl $name {
            $(
                $(#[$setter_doc])*
                pub fn $setter(&mut self, $param: $public, tween: AudioTween) {
                    let $param: $concrete = $param.into();
                    self.0.$setter($param, tween.into());
                }
            )*
        }
    };
}

effect_handle! {
    /// Controls a filter effect.
    FilterHandle(kira::effect::filter::FilterHandle) {
        /// Set the cutoff frequency of the filter (in hertz).
        set_cutoff(cutoff: f64 => f64);
        /// Set the resonance of the filter.
        set_resonance(resonance: f64 => f64);
        /// Set how much dry (unprocessed) signal is blended with the wet (processed) signal.
        set_mix(mix: impl Into<Mix> => Mix);
    }
}

impl FilterHandle {
    /// Set the frequencies that the filter will remove.
    pub fn set_mode(&mut self, mode: FilterMode) {
        self.0.set_mode(mode);
    }
}

effect_handle! {
    /// Controls a reverb effect.
    ReverbHandle(kira::effect::reverb::ReverbHandle) {
        /// Set how much the room reverberates.
        ///
        /// A higher value results in a bigger sounding room. `1.0` gives an infinitely
        /// reverberating room.
        set_feedback(feedback: f64 => f64);
        /// Set how quickly high frequencies disappear from the reverberation.
        set_damping(damping: f64 => f64);
        /// Set the stereo width of the reverb effect (`0.0` being fully mono, `1.0` fully stereo).
        set_stereo_width(stereo_width: f64 => f64);
        /// Set how much dry (unprocessed) signal is blended with the wet (processed) signal.
        set_mix(mix: impl Into<Mix> => Mix);
    }
}

effect_handle! {
    /// Controls a delay effect.
    DelayHandle(kira::effect::delay::DelayHandle) {
        /// Set the amount of feedback in Decibels.
        set_feedback(feedback: impl Into<Decibels> => Decibels);
        /// Set how much dry (unprocessed) signal is blended with the wet (processed) signal.
        set_mix(mix: impl Into<Mix> => Mix);
    }
}

effect_handle! {
    /// Controls a distortion effect.
    DistortionHandle(kira::effect::distortion::DistortionHandle) {
        /// Set how much the input signal is driven in Decibels.
        set_drive(drive: impl Into<Decibels> => Decibels);
        /// Set how much dry (unprocessed) signal is blended with the wet (processed) signal.
        set_mix(mix: impl Into<Mix> => Mix);
    }
}

impl DistortionHandle {
    /// Set the kind of distortion that is applied.
    pub fn set_kind(&mut self, kind: DistortionKind) {
        self.0.set_kind(kind);
    }
}

effect_handle! {
    /// Controls a compressor effect.
    CompressorHandle(kira::effect::compressor::CompressorHandle) {
        /// Set the volume above which volume will be decreased (in Decibels).
        set_threshold(threshold: f64 => f64);
        /// Set how much the signal will be compressed once it exceeds the threshold.
        set_ratio(ratio: f64 => f64);
        /// Set how quickly the compressor kicks in once the input exceeds the threshold.
        set_attack_duration(attack_duration: Duration => Duration);
        /// Set how quickly the compressor stops once the input falls below the threshold.
        set_release_duration(release_duration: Duration => Duration);
        /// Set the volume applied to the signal after compression (in Decibels).
        set_makeup_gain(makeup_gain: impl Into<Decibels> => Decibels);
        /// Set how much dry (unprocessed) signal is blended with the wet (processed) signal.
        set_mix(mix: impl Into<Mix> => Mix);
    }
}

effect_handle! {
    /// Controls an EQ filter effect.
    EqFilterHandle(kira::effect::eq_filter::EqFilterHandle) {
        /// Set the frequency the filter is centered on (in hertz).
        set_frequency(frequency: f64 => f64);
        /// Set the volume adjustment applied to the frequency range (in Decibels).
        set_gain(gain: impl Into<Decibels> => Decibels);
        /// Set the width of the frequency range that is adjusted.
        set_q(q: f64 => f64);
    }
}

impl EqFilterHandle {
    /// Set the shape of the frequency adjustment curve.
    pub fn set_kind(&mut self, kind: EqFilterKind) {
        self.0.set_kind(kind);
    }
}

effect_handle! {
    /// Controls a volume control effect.
    VolumeControlHandle(kira::effect::volume_control::VolumeControlHandle) {
        /// Set the volume in Decibels.
        set_volume(volume: impl Into<Decibels> => Decibels);
    }
}

effect_handle! {
    /// Controls a panning control effect.
    PanningControlHandle(kira::effect::panning_control::PanningControlHandle) {
        /// Set the panning.
        ///
        /// The default value is `0.0`. Values up to `1.0` pan to the right, values down to `-1.0`
        /// pan to the left.
        set_panning(panning: f32 => Panning);
    }
}

/// Implement [`AudioEffect`] for Kira's effect builders, wrapping the handles they return.
macro_rules! impl_audio_effect {
    ($($builder:ty => $handle:ident),* $(,)?) => {
        $(
            impl AudioEffect for $builder {
                type Handle = $handle;

                fn build_effect(self) -> (Box<dyn Effect>, Self::Handle) {
                    let (effect, handle) = KiraEffectBuilder::build(self);

                    (effect, $handle(handle))
                }
            }
        )*
    };
}

impl_audio_effect! {
    CompressorBuilder => CompressorHandle,
    DistortionBuilder => DistortionHandle,
    EqFilterBuilder => EqFilterHandle,
    FilterBuilder => FilterHandle,
    PanningControlBuilder => PanningControlHandle,
    ReverbBuilder => ReverbHandle,
    VolumeControlBuilder => VolumeControlHandle,
}
