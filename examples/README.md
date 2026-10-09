# Examples

These examples are simple Bevy Apps illustrating the capabilities of `bevy_kira_audio`. Run the examples with `cargo run --example <example>`.

| Example                                                  | Description                                                          |
|----------------------------------------------------------|----------------------------------------------------------------------|
| [`basic.rs`](/examples/basic.rs)                         | Display of basic functionality                                       |
| [`channel_control.rs`](/examples/channel_control.rs)     | Demonstrate controlling an audio channel                             |
| [`custom_channel.rs`](/examples/custom_channel.rs)       | How to add and use a custom audio channel                            |
| [`dynamic_channels.rs`](/examples/dynamic_channels.rs)   | Usage of dynamic audio channels                                      |
| [`effect_tail.rs`](/examples/effect_tail.rs)             | Effects ring out after a sound ends, but stop with it                |
| [`effects.rs`](/examples/effects.rs)                     | Effects on a single sound, changed at runtime                        |
| [`instance_control.rs`](/examples/instance_control.rs)   | Demonstrate controlling a single audio instance                      |
| [`multiple_channels.rs`](/examples/multiple_channels.rs) | GUI application with full control over tree different audio channels |
| [`peak_meter.rs`](/examples/peak_meter.rs)               | A custom effect measuring loudness to drive a UI meter               |
| [`reverb_channel.rs`](/examples/reverb_channel.rs)       | Reverb on a channel, applied to all its sounds and toggled at runtime |
| [`settings.rs`](/examples/settings.rs)                   | Demonstrate settings supported when playing a sound                  |
| [`settings_loader.rs`](/examples/settings_loader.rs)     | Loading a sound with applied settings                                |
| [`spatial.rs`](/examples/spatial.rs)                     | Demonstration of the limited support for spatial audio               |
| [`stacked_effects.rs`](/examples/stacked_effects.rs)     | A per-sound effect and a channel effect applied to the same sound    |
| [`status.rs`](/examples/status.rs)                       | Continuously get the playback state of a sound                       |
| [`stress_test.rs`](/examples/stress_test.rs)             | Example app playing a high number of sounds every frame              |

## Credits
The examples include third party assets:

Loop audio: [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/) [Jay_You](https://freesound.org/people/Jay_You/sounds/460432/)
