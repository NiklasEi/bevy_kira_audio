#[cfg(feature = "mp3")]
use anyhow::Result;
#[cfg(feature = "mp3")]
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
#[cfg(feature = "mp3")]
use bevy::utils::BoxedFuture;
#[cfg(feature = "mp3")]
use kira::sound::{Sound, SoundSettings};
#[cfg(feature = "mp3")]
use std::io::Cursor;

#[cfg(feature = "mp3")]
use crate::source::AudioSource;

#[derive(Default)]
pub struct Mp3Loader;

#[cfg(feature = "mp3")]
impl AssetLoader for Mp3Loader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let sound = Sound::from_mp3_reader(Cursor::new(bytes), SoundSettings::default())?;
            load_context.set_default_asset(LoadedAsset::new(AudioSource { sound }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mp3"]
    }
}
