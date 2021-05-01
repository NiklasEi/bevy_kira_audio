#[cfg(feature = "ogg")]
use anyhow::Result;
#[cfg(feature = "ogg")]
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
#[cfg(feature = "ogg")]
use bevy::utils::BoxedFuture;
#[cfg(feature = "ogg")]
use kira::sound::{Sound, SoundSettings};
#[cfg(feature = "ogg")]
use std::io::Cursor;

#[cfg(feature = "ogg")]
use crate::source::AudioSource;

#[derive(Default)]
pub struct OggLoader;

#[cfg(feature = "ogg")]
impl AssetLoader for OggLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let sound = Sound::from_ogg_reader(Cursor::new(bytes), SoundSettings::default())?;
            load_context.set_default_asset(LoadedAsset::new(AudioSource { sound }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ogg", "oga", "spx"]
    }
}
