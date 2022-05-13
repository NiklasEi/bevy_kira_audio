/// A channel to play audio in
///
/// You can play audio streams in this channel
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct AudioStreamChannel {
    key: String,
}

impl Default for AudioStreamChannel {
    fn default() -> Self {
        AudioStreamChannel {
            key: "default_channel".to_string(),
        }
    }
}

impl AudioStreamChannel {
    /// Create a new AudioChannel
    pub fn new(key: String) -> Self {
        AudioStreamChannel { key }
    }
}
