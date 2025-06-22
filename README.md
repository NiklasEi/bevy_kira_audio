# Bevy Kira audio

[![Crates.io](https://img.shields.io/crates/v/bevy_kira_audio.svg)](https://crates.io/crates/bevy_kira_audio)
[![docs](https://docs.rs/bevy_kira_audio/badge.svg)](https://docs.rs/bevy_kira_audio)
[![license](https://img.shields.io/crates/l/bevy_kira_audio)](https://github.com/NiklasEi/bevy_kira_audio#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_kira_audio.svg)](https://crates.io/crates/bevy_kira_audio)

This bevy plugin is intended to test an integration of [Kira][kira] into Bevy. The goal is to replace or update `bevy_audio`, if Kira turns out to be a good approach. Currently, this plugin can play `ogg`, `mp3`, `flac`, and `wav` formats and supports web builds.

Sound can be played in channels. Each channel has controls to pause or stop playback and can change the volume, playback speed, and panning of all sounds playing in it. You can easily add new channels and access them through Bevy's ECS (see the [`custom_channel` example](examples/custom_channel.rs)).

## Usage

*Note: the Bevy feature `bevy_audio` is enabled by default and not compatible with this plugin. Make sure to not have the `bevy_audio` feature enabled if you want to use `bevy_kira_audio`. The same goes for Bevy's `vorbis` feature. See [Bevys' Cargo file][bevy_default_features] for a list of all default features of version `0.16` and list them manually in your Cargo file excluding the ones you do not want. Make sure to set `default-features` to `false` for the Bevy dependency. You can take a look at [bevy_game_template's cargo file as an example](https://github.com/NiklasEi/bevy_game_template/blob/main/Cargo.toml).*


To play audio, you usually want to load audio files as assets. This requires `AssetLoaders`. `bevy_kira_audio` comes with loaders for most common audio formats. You can enable them with the features `ogg` (enabled by default), `mp3`, `wav`, or `flac`. The following example assumes that the feature `ogg` is enabled.

```rust no_run
use bevy_kira_audio::prelude::*;
use bevy::prelude::*;

fn main() {
   App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, start_background_audio)
        .run();
}

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("background_audio.ogg")).looped();
}
```

You can change settings like volume, panning, or playback rate for running sounds, or when starting to play a sound.
All changes can be done as smooth transitions. By default, they will be almost instantaneous.

### Sound settings

You can configure a sound when playing it:
```rust
use bevy_kira_audio::prelude::*;
use bevy::prelude::*;
use std::time::Duration;

fn play_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("background_audio.ogg"))
        // The first 0.5 seconds will not be looped and are the "intro"
        .loop_from(0.5)
        // Fade-in with a dynamic easing
        .fade_in(AudioTween::new(Duration::from_secs(2), AudioEasing::OutPowi(2)))
        // Only play on our right ear
        .with_panning(kira::Panning(1.0))
        // Increase playback rate by 50% (this also increases the pitch)
        .with_playback_rate(1.5)
        // Play at half volume
        .with_volume(0.5)
        // play the track reversed
        .reverse();
}
```

Optionally, you can also load a sound with already applied settings. This requires the feature `settings_loader`.

Sounds are configured in `ron` files. The following file loads as a `AudioSource` which is looped and has a 3 seconds intro before the loop:
```ron
(
    // The actual sound file in your assets directory
    file: "sounds/loop.ogg",

    loop_behavior: Some(3.0),
)
```
would make the loaded sound loop by default and start each repeated playback three seconds into the sound (the three seconds are the intro).

More settings are available. See the [`settings_loader` example](examples/settings_loader.rs) for all options.

### Controlling sounds

You can either control a whole audio channel and all instances playing in it ([`channel_control` example](examples/channel_control.rs)), or a single audio instance ([`instance_control` example](examples/instance_control.rs)). Both ways offer audio transitions with Tweens supporting multiple easings.

### Spatial audio

There is limited spatial audio support. Currently, only the volume of audio and it's panning can be automatically changed based on emitter and receiver positions. Take a look at the [`spatial` example](examples/spatial.rs) for some code.

## Compatible Bevy versions

The main branch is compatible with the latest Bevy release.

Compatibility of `bevy_kira_audio` versions:

| Bevy version | `bevy_kira_audio` version |
|:-------------|:--------------------------|
| `0.16`       | `0.23`                    |
| `0.15`       | `0.21` - `0.22`           |
| `0.14`       | `0.20`                    |
| `0.13`       | `0.19`                    |
| `0.12`       | `0.18`                    |
| `0.11`       | `0.16` - `0.17`           |
| `0.10`       | `0.15`                    |
| `0.9`        | `0.13` - `0.14`           |
| `0.8`        | `0.11` - `0.12`           |
| `0.7`        | `0.9` - `0.10`            |
| `0.6`        | `0.8`                     |
| `0.5`        | `0.4` - `0.7`             |
| `0.4`        | `0.3`                     |
| `0.15`       | `main`                    |
| `main`       | `bevy_main`               |

## License

Dual-licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

Assets in the examples might be distributed under different terms. See the [readme](examples/README.md#credits) in the `examples` directory.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.



[kira]: https://github.com/tesselode/kira
[bevy_default_features]: https://github.com/bevyengine/bevy/blob/v0.15.0/Cargo.toml#L101-L138
