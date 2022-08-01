# Changelog

## v0.11.0
- Fix channel playback states ([#54](https://github.com/NiklasEi/bevy_kira_audio/issues/54))
- Update to Bevy 0.8

## v0.10.0
- Allow configuring the audio backend through a settings resource
- Add support to load sound with any eligible settings (see [the example](examples/settings_loader.rs))
- Update to Kira version 0.6 ([#48](https://github.com/NiklasEi/bevy_kira_audio/issues/48))
- Make channels resources ([#43](https://github.com/NiklasEi/bevy_kira_audio/issues/43))

## v0.9.0
- Update to Bevy version 0.7
- the sound field in `AudioSource` is now public ([#37](https://github.com/NiklasEi/bevy_kira_audio/pull/37))

## v0.8.0
- Update to Bevy version 0.6

## v0.7.0
- The playback position of audio can be requested from the `Audio` resource
- Update to Rust edition 2021
- Removed direct dependencies on bevy sub crates

## v0.6.0
- Relicense under dual MIT or Apache-2.0
- Clean up stopped instances
- "ogg" is now a default feature 
- No longer panic when no Audio device can be found (analogue to bevy/audio)
- Files can be loaded with a semantic duration (see [the example](examples/settings_loader))
- The plugin will no longer compile if none of the features "mp3", "ogg", "wav", or "flac" are set
- Allow playing looped sounds with an intro
