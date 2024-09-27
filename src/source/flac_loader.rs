use anyhow::Result;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use kira::sound::static_sound::StaticSoundData;
use kira::sound::FromFileError;
use std::io::Cursor;
use thiserror::Error;

use crate::source::AudioSource;

/// Possible errors that can be produced by [`FlacLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum FlacLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// An Error loading sound from a file. See [`FromFileError`]
    #[error("Error while loading a sound: {0}")]
    FileError(#[from] FromFileError),
}

/// Asset loader for FLAC files.
#[derive(Default)]
pub struct FlacLoader;

impl AssetLoader for FlacLoader {
    type Asset = AudioSource;
    type Settings = ();
    type Error = FlacLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut sound_bytes = vec![];
        reader.read_to_end(&mut sound_bytes).await?;
        let sound = StaticSoundData::from_cursor(Cursor::new(sound_bytes))?;
        Ok(AudioSource { sound })
    }

    fn extensions(&self) -> &[&str] {
        &["flac"]
    }
}
