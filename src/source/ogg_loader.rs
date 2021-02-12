#[cfg(feature = "ogg")]
use anyhow::{Error, Result};
#[cfg(feature = "ogg")]
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
#[cfg(feature = "ogg")]
use bevy::utils::BoxedFuture;
#[cfg(feature = "ogg")]
use kira::sound::error::SoundFromFileError;
#[cfg(feature = "ogg")]
use kira::sound::{Sound, SoundSettings};
#[cfg(feature = "ogg")]
use kira::Frame;
#[cfg(feature = "ogg")]
use lewton::{inside_ogg::OggStreamReader, samples::Samples};
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
            let mut reader = OggStreamReader::new(Cursor::new(bytes))?;
            let mut stereo_samples = vec![];
            while let Some(packet) = reader.read_dec_packet_generic::<Vec<Vec<f32>>>()? {
                let num_channels = packet.len();
                let num_samples = packet.num_samples();
                match num_channels {
                    1 => {
                        for i in 0..num_samples {
                            stereo_samples.push(Frame::from_mono(packet[0][i]));
                        }
                    }
                    2 => {
                        for i in 0..num_samples {
                            stereo_samples.push(Frame::new(packet[0][i], packet[1][i]));
                        }
                    }
                    _ => {
                        return Err(Error::from(
                            SoundFromFileError::UnsupportedChannelConfiguration,
                        ))
                    }
                }
            }

            load_context.set_default_asset(LoadedAsset::new(AudioSource {
                sound: Sound::from_frames(
                    reader.ident_hdr.audio_sample_rate,
                    stereo_samples,
                    SoundSettings::default(),
                ),
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ogg"]
    }
}
