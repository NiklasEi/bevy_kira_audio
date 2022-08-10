# Bevy Kira audio

[![Crates.io](https://img.shields.io/crates/v/bevy_kira_audio.svg)](https://crates.io/crates/bevy_kira_audio)
[![docs](https://docs.rs/bevy_kira_audio/badge.svg)](https://docs.rs/bevy_kira_audio)
[![license](https://img.shields.io/crates/l/bevy_kira_audio)](https://github.com/NiklasEi/bevy_kira_audio#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_kira_audio.svg)](https://crates.io/crates/bevy_kira_audio)

This bevy plugin is intended to test an integration of [Kira][kira] into Bevy. The goal is to replace or update `bevy_audio`, if Kira turns out to be a good approach. Currently, this plugin can play `ogg`, `mp3`, `flac`, and `wav` formats and supports web builds.

Sound can be played in channels. Each channel has controls to pause or stop playback and can change the volume, playback speed, and panning of all sounds playing in it. You can easily add new channels and access them through Bevy's ECS (see the [`custom_channel` example](examples/custom_channel.rs)).

## Usage

*Note: the Bevy feature `bevy_audio` is enabled by default and not compatible with this plugin. Make sure to not have the `bevy_audio` feature enabled if you want to use `bevy_kira_audio`. The same goes for Bevy's `vorbis` feature. See [Bevys' Cargo file](https://github.com/bevyengine/bevy/blob/v0.8.0/Cargo.toml#L27-L40) for a list of all default features of version `0.8` and list them manually in your Cargo file excluding the ones you do not want. Make sure to set `default-features` to `false` for the Bevy dependency. You can take a look at [bevy_game_template's cargo file as an example](https://github.com/NiklasEi/bevy_game_template/blob/main/Cargo.toml).*


To play audio, you usually want to load audio files as assets. This requires `AssetLoaders`. `bevy_kira_audio` comes with loaders for most common audio formats. You can enable them with the features `ogg` (enabled by default), `mp3`, `wav`, or `flac`. The following example assumes that the feature `ogg` is enabled.

```rust no_run
use bevy_kira_audio::prelude::*;
use bevy::prelude::*;

fn main() {
   App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(start_background_audio)
        .run();
}

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play_looped(asset_server.load("background_audio.ogg"));
}
```

### Sound with custom settings

*Requires feature `settings_loader`*

It is possible to load sounds with custom settings from `ron` files. A common example would be a loop with an intro. Loading a ron file like this:
```ron
(
    // The actual sound file in your assets directory
    file: "sounds/loop.ogg",

    loop_behavior: Some(3.0),
)
```
would make the loaded sound loop by default and start each repeated playback three seconds into the sound (the three seconds are the intro).

More settings are available. See the [`settings_loader` example](examples/settings_loader.rs) for all options.

## Current and planned features
- [x] play common audio formats
  - [x] `ogg`
  - [x] `mp3`
  - [x] `wav`
  - [x] `flac`
- [x] web support
  - There are some differences between browsers:
    - Chrome requires an interaction with the website to play audio (e.g. a button click). This issue can be resolved with [a script][audio-context-resuming] in your `index.html` file ([usage example][bevy-game-template-audio-context-resuming]).
    - Firefox: The audio might sound distorted (this could be related to overall performance; see [issue #9][issue-9])
- [x] pause/resume and stop tracks
- [x] play a track on repeat
- [x] control volume
- [x] control playback rate
- [ ] control pitch (no change in playback rate)
- [x] control panning
- [x] get the current status and position of a track (see the [`status` example](examples/status.rs))
- [ ] audio streaming

## Compatible Bevy versions

The main branch is compatible with the latest Bevy release.

Compatibility of `bevy_kira_audio` versions:
| `bevy_kira_audio` | `bevy` |
|  :--              |  :--   |
| `0.11`            | `0.8`  |
| `0.9` - `0.10`    | `0.7`  |
| `0.8`             | `0.6`  |
| `0.4` - `0.7`     | `0.5`  |
| `0.3`             | `0.4`  |
| `main`            | `0.8`  |
| `bevy_main`       | `main` |

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
[kira-license]: https://github.com/tesselode/kira/blob/main/license.md
[rodio]: https://github.com/RustAudio/rodio
[oicana]: https://github.com/NiklasEi/oicana
[oicana-audio]: https://github.com/NiklasEi/oicana/blob/master/crates/oicana_plugin/src/audio.rs
[issue-9]: https://github.com/NiklasEi/bevy_kira_audio/issues/9
[audio-context-resuming]: https://developers.google.com/web/updates/2018/11/web-audio-autoplay#moving-forward
[bevy-game-template-audio-context-resuming]: https://github.com/NiklasEi/bevy_game_template/blob/main/index.html#L27-L90
