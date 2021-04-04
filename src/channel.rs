/// A channel to play audio in
///
/// You can play audio in this channel and controll
/// properties like the volume, playback rate or panning
/// ```edition2018
/// # use bevy_kira_audio::{AudioChannel, Audio};
/// # use bevy::prelude::*;
///
/// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
///     let channel = AudioChannel::new("my-channel".to_owned());
///     audio.play_in_channel(asset_server.load("audio.mp3"), &channel);
/// }
/// ```
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct AudioChannel {
    key: String,
}

impl Default for AudioChannel {
    fn default() -> Self {
        AudioChannel {
            key: "default_channel".to_string(),
        }
    }
}

impl AudioChannel {
    /// Create a new AudioChannel
    ///
    /// ```edition2018
    /// # use bevy_kira_audio::{AudioChannel, Audio};
    /// # use bevy::prelude::*;
    ///
    /// fn my_system(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    ///     let channel = AudioChannel::new("my-channel".to_owned());
    ///     audio.play_in_channel(asset_server.load("audio.mp3"), &channel);
    /// }
    /// ```
    pub fn new(key: String) -> Self {
        AudioChannel { key }
    }
}
