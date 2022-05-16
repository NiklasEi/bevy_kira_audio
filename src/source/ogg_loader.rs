use anyhow::Result;
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::utils::BoxedFuture;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use std::io::Cursor;

use crate::source::AudioSource;

#[derive(Default)]
pub struct OggLoader;

impl AssetLoader for OggLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut sound_bytes = vec![];
            for byte in bytes {
                sound_bytes.push(*byte);
            }
            let sound = StaticSoundData::from_cursor(
                Cursor::new(sound_bytes),
                StaticSoundSettings::default(),
            )?;
            load_context.set_default_asset(LoadedAsset::new(AudioSource { sound }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ogg", "oga", "spx"]
    }
}
