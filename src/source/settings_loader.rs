use std::time::Duration;
use std::{io::Cursor, path::PathBuf};

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext, ReadAssetBytesError};
use bevy::utils::BoxedFuture;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::sound::{EndPosition, FromFileError, PlaybackPosition, PlaybackRate, Region};
use kira::tween::Tween;
use kira::Volume;
use serde::Deserialize;
use thiserror::Error;

use crate::AudioSource;

#[derive(Default)]
pub struct SettingsLoader;

/// Kira sound settings
///
/// This is used when loading from a *.{wav,mp3,ogg,flac}.ron file to override
/// the default [`StaticSoundSettings`].
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SoundSettings {
    /// Location of the sound file.
    file: PathBuf,

    /// The region of the sound that should be played.
    #[serde(default = "default_full_file")]
    pub playback_region: Region,

    /// The portion of the sound that should be looped.
    #[serde(default)]
    pub loop_region: Option<Region>,

    /// Amplitude multiplier
    ///
    /// If the channel you play the sound is configured, it will overwrite the volume here.
    #[serde(default = "default_one")]
    pub volume: f64,

    /// The playback rate of the sound.
    ///
    /// Changing the playback rate will change both the speed
    /// and the pitch of the sound.
    ///
    /// If the channel you play the sound is configured, it will overwrite the volume here.
    #[serde(default = "default_one")]
    pub playback_rate: f64,

    /// The panning of the sound, where 0 is hard left
    /// and 1 is hard right.
    ///
    /// If the channel you play the sound is configured, it will overwrite the volume here.
    #[serde(default = "default_panning")]
    pub panning: f64,

    /// Whether the sound should play in reverse.
    ///
    /// If set to `true`, the start position will be relative
    /// to the end of the sound.
    #[serde(default)]
    pub reverse: bool,

    /// An optional linear fade-in from silence.
    ///
    /// The [`u64`] value is the duration of the tween in milliseconds.
    #[serde(default)]
    pub fade_in_tween: Option<u64>,
}

fn default_full_file() -> Region {
    Region {
        start: PlaybackPosition::Samples(0),
        end: EndPosition::EndOfAudio,
    }
}

fn default_one() -> f64 {
    1.0
}

fn default_panning() -> f64 {
    0.5
}

impl From<SoundSettings> for StaticSoundSettings {
    fn from(settings: SoundSettings) -> Self {
        let mut static_sound_settings = StaticSoundSettings::new();

        static_sound_settings.playback_region = settings.playback_region;
        static_sound_settings.volume = Volume::from(settings.volume).into();
        static_sound_settings.playback_rate = PlaybackRate::from(settings.playback_rate).into();
        static_sound_settings.panning = settings.panning.into();
        static_sound_settings.reverse = settings.reverse;
        static_sound_settings.loop_region = settings.loop_region;
        static_sound_settings.fade_in_tween = settings.fade_in_tween.map(|micros| Tween {
            duration: Duration::from_micros(micros),
            ..Default::default()
        });

        static_sound_settings
    }
}

/// Possible errors that can be produced by [`SettingsLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SettingsLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// An Error loading sound from a file. See [`FromFileError`]
    #[error("Error while loading a sound: {0}")]
    FileError(#[from] FromFileError),
    /// Failed to read audio asset
    #[error("Error while loading audio asset: {0}")]
    ReadAssetError(#[from] ReadAssetBytesError),
    /// A [RON Error](serde_ron::error::SpannedError)
    #[error("Could not parse RON: {0}")]
    RonError(#[from] ron::error::SpannedError),
}

impl AssetLoader for SettingsLoader {
    type Asset = AudioSource;
    type Settings = ();
    type Error = SettingsLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let sound_settings: SoundSettings = ron::de::from_bytes(&bytes)?;
            let sound_bytes = load_context
                .read_asset_bytes(sound_settings.file.clone())
                .await?;

            let sound =
                StaticSoundData::from_cursor(Cursor::new(sound_bytes), sound_settings.into())?;

            Ok(AudioSource { sound })
        })
    }

    fn extensions(&self) -> &[&str] {
        &[
            #[cfg(feature = "mp3")]
            "mp3.ron",
            #[cfg(feature = "wav")]
            "wav.ron",
            #[cfg(feature = "flac")]
            "flac.ron",
            #[cfg(feature = "ogg")]
            "ogg.ron",
            #[cfg(feature = "ogg")]
            "oga.ron",
            #[cfg(feature = "ogg")]
            "spx.ron",
        ]
    }
}
