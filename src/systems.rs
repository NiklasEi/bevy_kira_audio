//! Systems for performing common simple actions.

use crate::{AudioControl, AudioChannel};
use bevy::prelude::Res;
use std::marker::{Send, Sync};

/// Stop the playback of the given channel.
pub fn stop_playback<T>(channel: Res<AudioChannel<T>>)
where
    T: Send + Sync + 'static,
{
    channel.stop();
}

/// Pause the playback of the given channel.
pub fn pause_playback<T>(channel: Res<AudioChannel<T>>)
where
    T: Send + Sync + 'static,
{
    channel.pause();
}

/// Resume the playback of the given channel.
pub fn resume_playback<T>(channel: Res<AudioChannel<T>>)
where
    T: Send + Sync + 'static,
{
    channel.resume();
}
