#[cfg(feature = "flac")]
use anyhow::{Error, Result};
#[cfg(feature = "flac")]
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
#[cfg(feature = "flac")]
use bevy::utils::BoxedFuture;
#[cfg(feature = "flac")]
use claxon::FlacReader;
#[cfg(feature = "flac")]
use kira::sound::error::SoundFromFileError;
#[cfg(feature = "flac")]
use kira::sound::{Sound, SoundSettings};
#[cfg(feature = "flac")]
use kira::Frame;

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
            let mut reader = FlacReader::new(bytes)?;
            let stream_info = reader.streaminfo();
            let mut stereo_samples = vec![];
            match reader.streaminfo().channels {
                1 => {
                    for sample in reader.samples() {
                        let sample = sample?;
                        stereo_samples.push(Frame::from_i32(
                            sample,
                            sample,
                            stream_info.bits_per_sample,
                        ));
                    }
                }
                2 => {
                    let mut iter = reader.samples();
                    while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
                        stereo_samples.push(Frame::from_i32(
                            left?,
                            right?,
                            stream_info.bits_per_sample,
                        ));
                    }
                }
                _ => {
                    return Err(Error::from(
                        SoundFromFileError::UnsupportedChannelConfiguration,
                    ))
                }
            }

            load_context.set_default_asset(LoadedAsset::new(AudioSource {
                sound: Sound::from_frames(
                    stream_info.sample_rate,
                    stereo_samples,
                    SoundSettings::default(),
                ),
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["flac"]
    }
}
