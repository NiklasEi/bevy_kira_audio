//! Systems for performing common simple actions.

use crate::{AudioChannel, AudioControl};
use bevy::prelude::{Res, Resource};

/// Stop the playback of the given channel.
pub fn stop_playback<T: Resource>(channel: Res<AudioChannel<T>>) {
    channel.stop();
}

/// Pause the playback of the given channel.
pub fn pause_playback<T: Resource>(channel: Res<AudioChannel<T>>) {
    channel.pause();
}

/// Resume the playback of the given channel.
pub fn resume_playback<T: Resource>(channel: Res<AudioChannel<T>>) {
    channel.resume();
}
