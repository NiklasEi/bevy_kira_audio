#[cfg(feature = "wav")]
use anyhow::Result;
#[cfg(feature = "wav")]
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
#[cfg(feature = "wav")]
use bevy::utils::BoxedFuture;
#[cfg(feature = "wav")]
use kira::sound::{Sound, SoundSettings};
#[cfg(feature = "wav")]
use std::io::Cursor;

#[cfg(feature = "wav")]
use crate::source::AudioSource;

#[derive(Default)]
pub struct WavLoader;

#[cfg(feature = "wav")]
impl AssetLoader for WavLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let sound = Sound::from_wav_reader(Cursor::new(bytes), SoundSettings::default())?;
            load_context.set_default_asset(LoadedAsset::new(AudioSource { sound }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wav"]
    }
}
