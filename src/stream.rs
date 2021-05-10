use crate::AudioChannel;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Frame {
    pub left: f32,
    pub right: f32,
}

impl Into<kira::Frame> for Frame {
    fn into(self) -> kira::Frame {
        kira::Frame {
            left: self.left,
            right: self.right,
        }
    }
}

pub trait AudioStream: Debug + Send + Sync + 'static {
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

pub struct StreamedAudio<T: AudioStream> {
    pub(crate) commands: RwLock<VecDeque<(T, AudioChannel)>>,
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
    pub fn stream(&self, stream: T) {
        self.commands
            .write()
            .push_front((stream, AudioChannel::default()));
    }
}
