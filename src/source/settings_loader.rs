#[cfg(feature = "settings_loader")]
use std::{io::Cursor, path::PathBuf};

#[cfg(feature = "settings_loader")]
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
#[cfg(feature = "settings_loader")]
use bevy::utils::BoxedFuture;
#[cfg(feature = "settings_loader")]
use kira::sound::{error::SoundFromFileError, Sound, SoundSettings};
#[cfg(feature = "settings_loader")]
use serde::Deserialize;

#[cfg(feature = "settings_loader")]
use crate::AudioSource;

#[derive(Default)]
pub struct SettingsLoader;

#[cfg(feature = "settings_loader")]
fn default_settings_value() -> f64 {
    f64::MIN
}

/// Kira sound settings specification
///
/// This is used when loading from a *.{wav,mp3,ogg,flac}.ron file to override
/// the default `kira::SoundSettings`.
#[cfg(feature = "settings_loader")]
#[derive(Deserialize)]
struct SoundSettingsSpec {
    /// Location of the sound file.
    file: PathBuf,

    /// How long the sound is musically.
    ///
    /// For example, a recording of a 2-bar drum fill
    /// in an echoey cathedral may have 5 seconds of actual
    /// drumming and then 10 seconds of reverberations from
    /// the building. So even though the audio is 15 seconds
    /// long, you might say the music only lasts for 5 seconds.
    ///
    /// If set, the semantic duration of the sound will be
    /// used as the default end point when looping the sound.
    #[serde(default = "default_settings_value")]
    semantic_duration: f64,
}

#[cfg(feature = "settings_loader")]
fn positive_or_none(value: f64) -> Option<f64> {
    if value >= 0. {
        Some(value)
    } else {
        None
    }
}

#[cfg(feature = "settings_loader")]
fn load_sound(
    bytes: Vec<u8>,
    sound_settings: SoundSettingsSpec,
) -> Result<Sound, SoundFromFileError> {
    let settings = SoundSettings {
        semantic_duration: positive_or_none(sound_settings.semantic_duration),

        ..SoundSettings::default()
    };

    if let Some(extension) = sound_settings.file.as_path().extension() {
        match extension.to_str() {
            #[cfg(feature = "mp3")]
            Some("mp3") => return Sound::from_mp3_reader(Cursor::new(bytes), settings),
            #[cfg(feature = "ogg")]
            Some("ogg") => return Sound::from_ogg_reader(Cursor::new(bytes), settings),
            #[cfg(feature = "flac")]
            Some("flac") => return Sound::from_flac_reader(Cursor::new(bytes), settings),
            #[cfg(feature = "wav")]
            Some("wav") => return Sound::from_wav_reader(Cursor::new(bytes), settings),
            _ => {}
        }
    }

    Err(SoundFromFileError::UnsupportedAudioFileFormat)
}

#[cfg(feature = "settings_loader")]
impl AssetLoader for SettingsLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let sound_settings: SoundSettingsSpec = ron::de::from_bytes(bytes)?;
            let sound_bytes = load_context.read_asset_bytes(&sound_settings.file).await?;

            let sound = load_sound(sound_bytes, sound_settings)?;

            load_context.set_default_asset(LoadedAsset::new(AudioSource { sound }));

            Ok(())
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
