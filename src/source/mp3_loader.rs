#[cfg(feature = "mp3")]
use anyhow::{Error, Result};
#[cfg(feature = "mp3")]
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
#[cfg(feature = "mp3")]
use bevy::utils::BoxedFuture;
#[cfg(feature = "mp3")]
use kira::sound::error::SoundFromFileError;
#[cfg(feature = "mp3")]
use kira::sound::{Sound, SoundSettings};
#[cfg(feature = "mp3")]
use kira::Frame;

#[cfg(feature = "mp3")]
use minimp3;

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
            let mut decoder = minimp3::Decoder::new(bytes);
            let mut sample_rate = None;
            let mut stereo_samples = vec![];
            loop {
                match decoder.next_frame() {
                    Ok(frame) => {
                        if let Some(sample_rate) = sample_rate {
                            if sample_rate != frame.sample_rate {
                                return Err(Error::from(
                                    SoundFromFileError::UnsupportedAudioFileFormat,
                                ));
                            }
                        } else {
                            sample_rate = Some(frame.sample_rate);
                        }
                        match frame.channels {
                            1 => {
                                for sample in frame.data {
                                    stereo_samples.push(Frame::from_i32(
                                        sample.into(),
                                        sample.into(),
                                        16,
                                    ))
                                }
                            }
                            2 => {
                                let mut iter = frame.data.iter();
                                while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
                                    stereo_samples.push(Frame::from_i32(
                                        (*left).into(),
                                        (*right).into(),
                                        16,
                                    ))
                                }
                            }
                            _ => {
                                return Err(Error::from(
                                    SoundFromFileError::UnsupportedChannelConfiguration,
                                ))
                            }
                        }
                    }
                    Err(error) => match error {
                        minimp3::Error::Eof => break,
                        error => return Err(error.into()),
                    },
                }
            }
            let sample_rate = match sample_rate {
                Some(sample_rate) => sample_rate,
                None => return Err(Error::from(SoundFromFileError::UnsupportedAudioFileFormat)),
            };

            load_context.set_default_asset(LoadedAsset::new(AudioSource {
                sound: Sound::from_frames(
                    sample_rate as u32,
                    stereo_samples,
                    SoundSettings::default(),
                ),
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mp3"]
    }
}
