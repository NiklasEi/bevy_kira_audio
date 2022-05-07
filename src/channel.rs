/// A channel to play audio in
///
/// You can play audio streams in this channel and control
/// properties like the volume, playback rate or panning
/// ```edition2018
/// # use bevy_kira_audio::{AudioStreamChannel, Audio};
/// # use bevy::prelude::*;
///
/// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
///     let channel = AudioStreamChannel::new("my-channel".to_owned());
///     audio.play_in_channel(asset_server.load("audio.mp3"), &channel);
/// }
/// ```
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
    ///
    /// ```edition2018
    /// # use bevy_kira_audio::{AudioStreamChannel, Audio};
    /// # use bevy::prelude::*;
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     let channel = AudioStreamChannel::new("my-channel".to_owned());
    ///     audio.play_in_channel(asset_server.load("audio.mp3"), &channel);
    /// }
    /// ```
    pub fn new(key: String) -> Self {
        AudioStreamChannel { key }
    }
}
