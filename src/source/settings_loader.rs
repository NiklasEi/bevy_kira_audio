use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, ReadAssetBytesError};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::sound::{FromFileError, PlaybackPosition, Region};
use kira::{Decibels, PlaybackRate, Tween};
use serde::Deserialize;
use std::time::Duration;
use std::{io::Cursor, path::PathBuf};
use thiserror::Error;

use crate::AudioSource;

/// Asset loader for sound settings files.
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

    /// The second from which the sound should be started.
    #[serde(default)]
    pub start_position: f64,

    /// The portion of the sound that should be looped.
    #[serde(default)]
    pub loop_region: Option<Region>,

    /// Amplitude multiplier
    ///
    /// If the channel you play the sound is configured, it will overwrite the volume here.
    #[serde(default = "default_one")]
    pub volume: f32,

    /// The playback rate of the sound.
    ///
    /// Changing the playback rate will change both the speed
    /// and the pitch of the sound.
    ///
    /// If the channel you play the sound is configured, it will overwrite the volume here.
    #[serde(default = "default_one_f64")]
    pub playback_rate: f64,

    /// The panning of the sound, where 0 is hard left
    /// and 1 is hard right.
    ///
    /// If the channel you play the sound is configured, it will overwrite the volume here.
    #[serde(default = "default_panning")]
    pub panning: kira::Panning,

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

fn default_one() -> f32 {
    1.0
}
fn default_one_f64() -> f64 {
    1.0
}

fn default_panning() -> kira::Panning {
    kira::Panning(0.0)
}

impl From<SoundSettings> for StaticSoundSettings {
    fn from(settings: SoundSettings) -> Self {
        let mut static_sound_settings = StaticSoundSettings::new();

        static_sound_settings.start_position = PlaybackPosition::Seconds(settings.start_position);
        static_sound_settings.volume = Decibels::from(settings.volume).into();
        static_sound_settings.playback_rate = PlaybackRate::from(settings.playback_rate).into();
        static_sound_settings.panning = kira::Value::Fixed(settings.panning);
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

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let sound_settings: SoundSettings = ron::de::from_bytes(&bytes)?;
        let sound_bytes = load_context
            .read_asset_bytes(sound_settings.file.clone())
            .await?;

        let mut sound = StaticSoundData::from_cursor(Cursor::new(sound_bytes))?;
        sound.settings = sound_settings.into();

        Ok(AudioSource { sound })
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
