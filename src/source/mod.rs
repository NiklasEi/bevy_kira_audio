//! Asset loaders for commonly used audio file formats

#[cfg(feature = "flac")]
pub mod flac_loader;
#[cfg(feature = "mp3")]
pub mod mp3_loader;
#[cfg(feature = "ogg")]
pub mod ogg_loader;
#[cfg(feature = "settings_loader")]
pub mod settings_loader;
#[cfg(feature = "wav")]
pub mod wav_loader;

use bevy::reflect::{TypePath, TypeUuid};
use kira::sound::static_sound::StaticSoundData;

/// A source of audio data
#[derive(Clone, TypeUuid, TypePath)]
#[uuid = "6a9fc4ca-b5b5-94d6-613c-522e2d9fe86d"]
pub struct AudioSource {
    /// The Kira sound making up this `AudioSource`
    pub sound: StaticSoundData,
}
