#[cfg(feature = "spec")]
use std::{io::Cursor, path::PathBuf};

#[cfg(feature = "spec")]
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
#[cfg(feature = "spec")]
use bevy_utils::BoxedFuture;
#[cfg(feature = "spec")]
use kira::sound::{error::SoundFromFileError, Sound, SoundSettings};
#[cfg(feature = "spec")]
use serde::Deserialize;

#[cfg(feature = "spec")]
use crate::AudioSource;

#[derive(Default)]
pub struct SpecLoader;

/// Kira sound settigns specification
///
/// This is used when loading from a *.{wav,mp3,ogg,flac}.ron file to override
/// the default `kira::SoundSettings`.
#[cfg(feature = "spec")]
#[derive(Deserialize)]
struct SoundSettingsSpec {
    /// Location of the sound file.
    file: PathBuf,

    /// Whether the sound should have a "cool off" period after playing
    /// before it can be played again, and if so, the duration
    /// of that cool off period.
    ///
    /// This is useful to avoid situations where the same sound
    /// is played multiple times at the exact same point in time,
    /// resulting in the sound being louder than normal.
    cooldown: Option<f64>,

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
    semantic_duration: Option<f64>,

    /// Whether the sound should be looped by default, and if so,
    /// the point an instance should jump back to when it reaches
    /// the end.
    default_loop_start: Option<f64>,
}

#[cfg(feature = "spec")]
fn load_sound(
    bytes: Vec<u8>,
    sound_settings: SoundSettingsSpec,
) -> Result<Sound, SoundFromFileError> {
    let settings = SoundSettings {
        cooldown: sound_settings.cooldown,
        semantic_duration: sound_settings.semantic_duration,
        default_loop_start: sound_settings.default_loop_start,

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

#[cfg(feature = "spec")]
impl AssetLoader for SpecLoader {
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
