# Bevy Kira audio

This bevy plugin is intended for testing of improvements for `bevy_audio`. The biggest change so far from the original crate has been a switch from [Rodio][rodio] to [Kira][kira].

I am using [Oicana][oicana] as "guinea pig project" and will keep it's [game audio plugin][oicana-audio] up to date with this crate.

## Usage
To initialize the corresponding `AssetLoaders`, use at least one of the features `mp3`, `ogg`, `wav`, or `flac`. The following example assumes that `bevy_kira_audio/mp3` is used.

```rust
use bevy_kira_audio::{Audio, AudioPlugin};

// in your game's AppBuilder:
// app.add_plugin(AudioPlugin)

fn my_audio_system(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    let music_handle = asset_server.get_handle("sounds/music.mp3");
    audio.play(music_handle);
}
```

## To do
- [ ] pause/resume and stop tracks
- [ ] play a track on repeat
- [x] play other formats than `mp3`
  - [x] `ogg`
  - [x] `wav`
  - [x] `flac`
- [ ] get the current status of a track (time elapsed/left)?
- [ ] web support

## License

This crate is distributed under the terms of the [MIT license](LICENSE.md).

Substantial parts of the asset loaders were taken from [Kira][kira], under the [MIT license][kira-license] (`Copyright 2020 Andrew Minnich`).



[kira]: https://github.com/tesselode/kira
[kira-license]: https://github.com/tesselode/kira/blob/main/license.md
[rodio]: https://github.com/RustAudio/rodio
[oicana]: https://github.com/NiklasEi/oicana
[oicana-audio]: https://github.com/NiklasEi/oicana/blob/master/crates/oicana_plugin/src/audio.rs
