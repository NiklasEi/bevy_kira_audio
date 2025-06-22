# Changelog
## v0.24.0 - 21.06.2025
- Depend on Kira `0.10.8`

## v0.23.0 - 26.04.2025
- Update to Bevy `0.16`

## v0.22.0 - 05.01.2025
- Update Kira to 0.9
  - Removed `playback_region` from `SoundSettings`
- Add `android_shared_stdcxx` feature for Android Builds
- Fix spatial audio when position of receiver and emitter are the same ([#135](https://github.com/NiklasEi/bevy_kira_audio/issues/135))
- Add `SpatialRadius` to control spatial audio distance range per entity
- Rename `SpatialAudio` to `GlobalSpatialRadius` and rename its field `max_distance` to `radius`
- Add `SpatialDampingCurve` component to control the damping of spatial audio over distance using Bevy easing functions

## v0.21.0 - 30.11.2024
- Update to Bevy `0.15`

## v0.20.0 - 04.07.2024
- Update to Bevy `0.14`
- Asset loaders are now public
- Fix typos `spacial` -> `spatial`

## v0.19.0 - 17.02.2024
- Update to Bevy `0.13`

## v0.18.0 - 04.11.2023
- Update to Bevy `0.12`

## v0.17.0
- Multiply instance volume with channel volume ([#103](https://github.com/NiklasEi/bevy_kira_audio/issues/103))
- Allow playing a paused sound using `.play(...).paused()` ([#105](https://github.com/NiklasEi/bevy_kira_audio/issues/105))
- Configure the end of the loop region (by @Tracreed in [#104](https://github.com/NiklasEi/bevy_kira_audio/pull/104))

## v0.16.0
- Update to Bevy `0.11`
- Spatial audio improvements ([#94](https://github.com/NiklasEi/bevy_kira_audio/pull/94))
  - Check if receiver exists before updating
  - Cleanup instances from spatial audio
- Offer iterator over dynamic channel keys and values ([#69](https://github.com/NiklasEi/bevy_kira_audio/issues/69))
- Allow resuming while pausing or stopping ([#98](https://github.com/NiklasEi/bevy_kira_audio/pull/98))

## v0.15.0
- Update to Bevy `0.10`
- Fix: stop spatial audio from getting louder again at large distances ([#88](https://github.com/NiklasEi/bevy_kira_audio/issues/88))

## v0.14.0
- Limited support for spatial audio
  - Add `Emitter` and `Receiver` components to spatial entities
  - Audio volume and panning will be automatically changed
  - New example [`spatial.rs`](examples/spatial.rs)
- Support for setting volume in Decibels ([#81](https://github.com/NiklasEi/bevy_kira_audio/issues/81))
- Reexport some used Kira types ([#73](https://github.com/NiklasEi/bevy_kira_audio/issues/73))

## v0.13.0
- Update to Bevy 0.9

## v0.12.0
- Support changing most settings when playing a sound ([#65](https://github.com/NiklasEi/bevy_kira_audio/issues/65))
  - Removed `.play_looped(handle)` in preference for `.play(handle).looped()`
- Directly control single audio instances ([#53](https://github.com/NiklasEi/bevy_kira_audio/issues/53))
  - When playing a sound you get an asset handle back (see new [instance_control example](/examples/instance_control.rs))
  - This also adds `seek` controls to bevy_kira_audio
- Dynamic audio channels ([#66](https://github.com/NiklasEi/bevy_kira_audio/issues/66))
- All audio commands can use smooth transitions with configurable Tweens ([#65](https://github.com/NiklasEi/bevy_kira_audio/issues/65))
- Stop commands will now also stop queued sounds ([#62](https://github.com/NiklasEi/bevy_kira_audio/issues/62))
- `is_playing_sound` method to quickly determine if a channel is in use at the moment ([#55](https://github.com/NiklasEi/bevy_kira_audio/issues/55))
- No more panics, if Kira's audio command que is full ([#51](https://github.com/NiklasEi/bevy_kira_audio/issues/51))

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
- Files can be loaded with a semantic duration (see [the example](examples/settings_loader.rs))
- The plugin will no longer compile if none of the features "mp3", "ogg", "wav", or "flac" are set
- Allow playing looped sounds with an intro
