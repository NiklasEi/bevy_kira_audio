use std::time::Duration;
use std::{io::Cursor, path::PathBuf};

use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::utils::BoxedFuture;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::sound::{EndPosition, PlaybackPosition, PlaybackRate, Region};
use kira::tween::Tween;
use kira::Volume;
use serde::Deserialize;

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

impl AssetLoader for SettingsLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let sound_settings: SoundSettings = ron::de::from_bytes(bytes)?;
            let sound_bytes = load_context.read_asset_bytes(&sound_settings.file).await?;

            let sound =
                StaticSoundData::from_cursor(Cursor::new(sound_bytes), sound_settings.into())?;

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
