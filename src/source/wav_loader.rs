#[cfg(feature = "wav")]
use anyhow::{Error, Result};
#[cfg(feature = "wav")]
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
#[cfg(feature = "wav")]
use bevy::utils::BoxedFuture;
#[cfg(feature = "wav")]
use hound::WavReader;
#[cfg(feature = "wav")]
use kira::sound::error::SoundFromFileError;
#[cfg(feature = "wav")]
use kira::sound::{Sound, SoundSettings};
#[cfg(feature = "wav")]
use kira::Frame;

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
            let mut reader = WavReader::new(bytes)?;
            let spec = reader.spec();
            let mut stereo_samples = vec![];
            match reader.spec().channels {
                1 => match spec.sample_format {
                    hound::SampleFormat::Float => {
                        for sample in reader.samples::<f32>() {
                            stereo_samples.push(Frame::from_mono(sample?))
                        }
                    }
                    hound::SampleFormat::Int => {
                        for sample in reader.samples::<i32>() {
                            let sample = sample?;
                            stereo_samples.push(Frame::from_i32(
                                sample,
                                sample,
                                spec.bits_per_sample.into(),
                            ));
                        }
                    }
                },
                2 => match spec.sample_format {
                    hound::SampleFormat::Float => {
                        let mut iter = reader.samples::<f32>();
                        while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
                            stereo_samples.push(Frame::new(left?, right?));
                        }
                    }
                    hound::SampleFormat::Int => {
                        let mut iter = reader.samples::<i32>();
                        while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
                            stereo_samples.push(Frame::from_i32(
                                left?,
                                right?,
                                spec.bits_per_sample.into(),
                            ));
                        }
                    }
                },
                _ => {
                    return Err(Error::from(
                        SoundFromFileError::UnsupportedChannelConfiguration,
                    ))
                }
            }

            load_context.set_default_asset(LoadedAsset::new(AudioSource {
                sound: Sound::from_frames(
                    reader.spec().sample_rate,
                    stereo_samples,
                    SoundSettings::default(),
                ),
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wav"]
    }
}
