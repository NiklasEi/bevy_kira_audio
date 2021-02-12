# Bevy Kira audio

This bevy plugin is intended for testing of improvements for `bevy_audio`. The biggest change so far from the original crate has been a switch from [Rodio](https://github.com/RustAudio/rodio) to [Kira](https://github.com/tesselode/kira).

I am using [Oicana](https://github.com/NiklasEi/oicana) as "guinea pig project" and will keep it's [game audio system](https://github.com/NiklasEi/oicana/blob/master/crates/oicana_plugin/src/audio.rs) up to date with this plugin.

## To do
- [ ] pause/resume and stop tracks
- [ ] play a track on repeat
- [ ] play other formats than `mp3`
  - [x] `ogg`
  - [x] `wav`
  - [ ] `flac`
- [ ] get the current status of a track (time elapsed/left)?
- [ ] web support
