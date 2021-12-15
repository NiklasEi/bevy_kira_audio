mod flac_loader;
mod mp3_loader;
mod ogg_loader;
mod settings_loader;
mod wav_loader;

use bevy::reflect::TypeUuid;
use kira::sound::Sound;

pub use flac_loader::FlacLoader;
pub use mp3_loader::Mp3Loader;
pub use ogg_loader::OggLoader;
pub use settings_loader::SettingsLoader;
pub use wav_loader::WavLoader;

/// A source of audio data
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "6a9fc4ca-b5b5-94d6-613c-522e2d9fe86d"]
pub struct AudioSource {
    pub(crate) sound: Sound,
}
