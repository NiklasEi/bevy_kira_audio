use anyhow::Result;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::sound::FromFileError;
use std::io::Cursor;
use thiserror::Error;

use crate::source::AudioSource;

/// Asset loader for MP3 files.
#[derive(Default)]
pub struct Mp3Loader;

/// Possible errors that can be produced by [`Mp3Loader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Mp3LoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// An Error loading sound from a file. See [`FromFileError`]
    #[error("Error while loading a sound: {0}")]
    FileError(#[from] FromFileError),
}

impl AssetLoader for Mp3Loader {
    type Asset = AudioSource;
    type Settings = ();
    type Error = Mp3LoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut sound_bytes = vec![];
        reader.read_to_end(&mut sound_bytes).await?;
        let sound =
            StaticSoundData::from_cursor(Cursor::new(sound_bytes), StaticSoundSettings::default())?;
        Ok(AudioSource { sound })
    }

    fn extensions(&self) -> &[&str] {
        &["mp3"]
    }
}
