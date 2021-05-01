#[cfg(feature = "flac")]
use anyhow::Result;
#[cfg(feature = "flac")]
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
#[cfg(feature = "flac")]
use bevy::utils::BoxedFuture;
#[cfg(feature = "flac")]
use kira::sound::{Sound, SoundSettings};
#[cfg(feature = "flac")]
use std::io::Cursor;

#[cfg(feature = "flac")]
use crate::source::AudioSource;

#[derive(Default)]
pub struct FlacLoader;

#[cfg(feature = "flac")]
impl AssetLoader for FlacLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let sound = Sound::from_flac_reader(Cursor::new(bytes), SoundSettings::default())?;
            load_context.set_default_asset(LoadedAsset::new(AudioSource { sound }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["flac"]
    }
}
