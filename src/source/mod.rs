mod flac_loader;
mod mp3_loader;
mod ogg_loader;
mod wav_loader;

use bevy_reflect::TypeUuid;
use kira::sound::Sound;

pub use flac_loader::FlacLoader;
pub use mp3_loader::Mp3Loader;
pub use ogg_loader::OggLoader;
pub use wav_loader::WavLoader;

/// A source of audio data
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "7a14806a-672b-443b-8d16-4f18afefa463"]
pub struct AudioSource {
    pub(crate) sound: Sound,
}
