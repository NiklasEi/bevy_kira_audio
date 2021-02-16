use crate::channel::ChannelId;
use crate::source::AudioSource;
use bevy::prelude::Handle;
use parking_lot::RwLock;
use std::collections::VecDeque;

pub enum AudioCommands {
    Play(PlayAudioSettings),
    Stop,
    Pause,
    Resume,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct PlayAudioSettings {
    pub source: Handle<AudioSource>,
    pub looped: bool,
}

#[derive(Default)]
pub struct Audio {
    pub commands: RwLock<VecDeque<(AudioCommands, ChannelId)>>,
}

impl Audio {
    pub fn play(&self, audio_source: Handle<AudioSource>) {
        self.commands.write().push_front((
            AudioCommands::Play(PlayAudioSettings {
                source: audio_source,
                looped: false,
            }),
            ChannelId::default(),
        ));
    }

    pub fn play_looped(&self, audio_source: Handle<AudioSource>) {
        self.commands.write().push_front((
            AudioCommands::Play(PlayAudioSettings {
                source: audio_source,
                looped: true,
            }),
            ChannelId::default(),
        ));
    }

    pub fn stop(&self) {
        self.commands
            .write()
            .push_front((AudioCommands::Stop, ChannelId::default()));
    }

    pub fn pause(&self) {
        self.commands
            .write()
            .push_front((AudioCommands::Pause, ChannelId::default()));
    }

    pub fn resume(&self) {
        self.commands
            .write()
            .push_front((AudioCommands::Resume, ChannelId::default()));
    }

    pub fn play_in_channel(&self, audio_source: Handle<AudioSource>, channel_id: ChannelId) {
        self.commands.write().push_front((
            AudioCommands::Play(PlayAudioSettings {
                source: audio_source,
                looped: false,
            }),
            channel_id,
        ));
    }

    pub fn play_looped_in_channel(&self, audio_source: Handle<AudioSource>, channel_id: ChannelId) {
        self.commands.write().push_front((
            AudioCommands::Play(PlayAudioSettings {
                source: audio_source,
                looped: true,
            }),
            channel_id,
        ));
    }

    pub fn stop_channel(&self, channel_id: ChannelId) {
        self.commands
            .write()
            .push_front((AudioCommands::Stop, channel_id));
    }

    pub fn pause_channel(&self, channel_id: ChannelId) {
        self.commands
            .write()
            .push_front((AudioCommands::Pause, channel_id));
    }

    pub fn resume_channel(&self, channel_id: ChannelId) {
        self.commands
            .write()
            .push_front((AudioCommands::Resume, channel_id));
    }
}
