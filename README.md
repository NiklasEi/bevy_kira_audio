# Bevy improved audio

This bevy plugin is intended as an easy way to test improvements for `bevy_audio`. The initial code is from the current implementation of `bevy_audio` and thus `Copyright (c) 2020 Carter Anderson`. You can find bevy's [MIT license here](https://github.com/bevyengine/bevy/blob/master/LICENSE) .

I am using [Oicana](https://github.com/NiklasEi/oicana) as "guinea pig project" and will keep it's [game audio system](https://github.com/NiklasEi/oicana/blob/master/crates/oicana_plugin/src/audio.rs) up to date with this plugin.

## To do
- [ ] pause/resume and stop tracks
- [ ] play a track on repeat
- play other formats than `mp3`
  - [x] `ogg`
  - [ ] `wav`
  - [ ] `flac`
- [ ] get the current status of a track (time elapsed/left)?
- [ ] web support
