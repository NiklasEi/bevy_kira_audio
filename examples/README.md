# Examples

These examples are simple Bevy Apps illustrating the capabilities of `bevy_kira_audio`. Run the examples with `cargo run --example <example>`.

| Example                                                  | Description                                                          |
|----------------------------------------------------------|----------------------------------------------------------------------|
| [`basic.rs`](/examples/basic.rs)                         | Display of basic functionality                                       |
| [`channel_control.rs`](/examples/channel_control.rs)     | Demonstrate controlling an audio channel                             |
| [`custom_channel.rs`](/examples/custom_channel.rs)       | How to add and use a custom audio channel                            |
| [`dynamic_channels.rs`](/examples/dynamic_channels.rs)   | Usage of dynamic audio channels                                      |
| [`filter.rs`](/examples/filter.rs)                       | Per-instance low-pass filter with runtime cutoff toggling            |
| [`filter_basic.rs`](/examples/filter_basic.rs)           | Simple low-pass filter using `with_effect` (no runtime control)      |
| [`instance_control.rs`](/examples/instance_control.rs)   | Demonstrate controlling a single audio instance                      |
| [`multiple_channels.rs`](/examples/multiple_channels.rs) | GUI application with full control over tree different audio channels  |
| [`reverb.rs`](/examples/reverb.rs)                       | Per-instance reverb with runtime on/off toggling                     |
| [`reverb_channel.rs`](/examples/reverb_channel.rs)       | Channel-level reverb applied to multiple sounds                      |
| [`settings.rs`](/examples/settings.rs)                   | Demonstrate settings supported when playing a sound                  |
| [`settings_loader.rs`](/examples/settings_loader.rs)     | Loading a sound with applied settings                                |
| [`spatial.rs`](/examples/spatial.rs)                     | Demonstration of the limited support for spatial audio               |
| [`status.rs`](/examples/status.rs)                       | Continuously get the playback state of a sound                       |
| [`stress_test.rs`](/examples/stress_test.rs)             | Example app playing a high number of sounds every frame              |

## Credits
The examples include third party assets:

Loop audio: [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/) [Jay_You](https://freesound.org/people/Jay_You/sounds/460432/)
