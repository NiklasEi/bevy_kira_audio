use crate::AudioChannel;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::fmt::Debug;

/// A single Frame of an audio stream
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Frame {
    /// Value of the left channel in the given frame
    pub left: f32,

    /// Value of the right channel in the given frame
    pub right: f32,
}

impl Frame {
    /// Creates a frame with the given left and right values
    pub fn new(left: f32, right: f32) -> Self {
        Self { left, right }
    }

    /// Creates a frame using the given value for left and right
    pub fn from_mono(value: f32) -> Self {
        Self::new(value, value)
    }
}

impl From<Frame> for kira::Frame {
    fn from(frame: Frame) -> Self {
        kira::Frame {
            left: frame.left,
            right: frame.right,
        }
    }
}

/// An audio stream producing frames
pub trait AudioStream: Debug + Send + Sync + 'static {
    /// Produce the next frame in the audio stream
    fn next(&mut self, dt: f64) -> Frame;
}

#[derive(Debug)]
pub(crate) struct InternalAudioStream<T: AudioStream> {
    input: T,
}

impl<T> InternalAudioStream<T>
where
    T: AudioStream,
{
    pub fn new(incoming_stream: T) -> Self {
        Self {
            input: incoming_stream,
        }
    }
}

impl<T> kira::audio_stream::AudioStream for InternalAudioStream<T>
where
    T: AudioStream,
{
    fn next(&mut self, dt: f64) -> kira::Frame {
        self.input.next(dt).into()
    }
}

pub enum StreamCommands<T: AudioStream> {
    Play(T),
    Stop,
}

/// Use this resource to start and stop audio streams
/// ```edition2018
/// # use bevy::prelude::*;
/// # use bevy_kira_audio::{AudioStream, Frame, StreamedAudio};
///
/// #[derive(Debug, Default)]
/// struct SineStream {
///     t: f64,
///     note: f64,
///     frequency: f64
/// }
///
/// impl AudioStream for SineStream {
///     fn next(&mut self, _: f64) -> Frame {
///         let increment = 2.0 * std::f64::consts::PI * self.note / self.frequency;
///         self.t += increment;
///
///         let sample: f64 = self.t.sin();
///         Frame {
///             left: sample as f32,
///             right: sample as f32,
///         }
///     }
/// }
///
/// fn start_stream_system(audio: Res<StreamedAudio<SineStream>>) {
///     audio.stream(SineStream { t: 0.0, note: 440.0, frequency: 44_000.0 });
/// }
/// ```
pub struct StreamedAudio<T: AudioStream> {
    pub(crate) commands: RwLock<VecDeque<(StreamCommands<T>, AudioChannel)>>,
}

impl<T> Default for StreamedAudio<T>
where
    T: AudioStream,
{
    fn default() -> Self {
        Self {
            commands: RwLock::new(VecDeque::default()),
        }
    }
}

impl<T> StreamedAudio<T>
where
    T: AudioStream,
{
    /// Start an audio stream in the default channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{StreamedAudio, AudioStream, Frame};
    ///
    /// #[derive(Debug, Default)]
    /// struct SineStream {
    ///     t: f64,
    ///     note: f64,
    ///     frequency: f64,
    /// }
    ///
    /// impl AudioStream for SineStream {
    ///     fn next(&mut self, _: f64) -> Frame {
    ///         self.t += 2.0 * std::f64::consts::PI * self.note / self.frequency;
    ///         Frame::from_mono(self.t.sin() as f32)
    ///     }
    /// }
    ///
    /// fn my_system(audio: Res<StreamedAudio<SineStream>>) {
    ///     audio.stream(SineStream { t: 0.0, note: 440.0, frequency: 44_000.0 });
    /// }
    /// ```
    pub fn stream(&self, stream: T) {
        self.commands
            .write()
            .push_front((StreamCommands::Play(stream), AudioChannel::default()));
    }

    /// Stop all audio streams in the default channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{StreamedAudio, AudioStream, Frame};
    ///
    /// #[derive(Debug, Default)]
    /// struct SineStream {
    ///     t: f64,
    ///     note: f64,
    ///     frequency: f64,
    /// }
    ///
    /// impl AudioStream for SineStream {
    ///     fn next(&mut self, _: f64) -> Frame {
    ///         self.t += 2.0 * std::f64::consts::PI * self.note / self.frequency;
    ///         Frame::from_mono(self.t.sin() as f32)
    ///     }
    /// }
    ///
    /// fn my_system(audio: Res<StreamedAudio<SineStream>>) {
    ///     audio.stop();
    /// }
    /// ```
    pub fn stop(&self) {
        self.commands
            .write()
            .push_front((StreamCommands::Stop, AudioChannel::default()));
    }

    /// Start an audio stream in the given channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{StreamedAudio, AudioStream, Frame, AudioChannel};
    ///
    /// #[derive(Debug, Default)]
    /// struct SineStream {
    ///     t: f64,
    ///     note: f64,
    ///     frequency: f64,
    /// }
    ///
    /// impl AudioStream for SineStream {
    ///     fn next(&mut self, _: f64) -> Frame {
    ///         self.t += 2.0 * std::f64::consts::PI * self.note / self.frequency;
    ///         Frame::from_mono(self.t.sin() as f32)
    ///     }
    /// }
    ///
    /// fn my_system(audio: Res<StreamedAudio<SineStream>>) {
    ///     audio.stream_in_channel(SineStream { t: 0.0, note: 440.0, frequency: 44_000.0 }, &AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn stream_in_channel(&self, stream: T, channel_id: &AudioChannel) {
        self.commands
            .write()
            .push_front((StreamCommands::Play(stream), channel_id.clone()));
    }

    /// Stop all audio streams in the given channel
    ///
    /// ```edition2018
    /// # use bevy::prelude::*;
    /// # use bevy_kira_audio::{StreamedAudio, AudioStream, Frame, AudioChannel};
    ///
    /// #[derive(Debug, Default)]
    /// struct SineStream {
    ///     t: f64,
    ///     note: f64,
    ///     frequency: f64,
    /// }
    ///
    /// impl AudioStream for SineStream {
    ///     fn next(&mut self, _: f64) -> Frame {
    ///         self.t += 2.0 * std::f64::consts::PI * self.note / self.frequency;
    ///         Frame::from_mono(self.t.sin() as f32)
    ///     }
    /// }
    ///
    /// fn my_system(audio: Res<StreamedAudio<SineStream>>) {
    ///     audio.stop_channel(&AudioChannel::new("my-channel".to_owned()));
    /// }
    /// ```
    pub fn stop_channel(&self, channel_id: &AudioChannel) {
        self.commands
            .write()
            .push_front((StreamCommands::Stop, channel_id.clone()));
    }
}
