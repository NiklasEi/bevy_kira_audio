use crate::channel::ChannelId;
use crate::source::AudioSource;
use bevy::prelude::Handle;
use parking_lot::RwLock;
use std::collections::VecDeque;

pub enum AudioCommands {
    Play(PlayAudioSettings),
    SetVolume(f32),
    SetPanning(f32),
    SetPitch(f32),
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

    pub fn set_volume(&self, volume: f32) {
        self.commands
            .write()
            .push_front((AudioCommands::SetVolume(volume), ChannelId::default()));
    }

    pub fn set_panning(&self, panning: f32) {
        self.commands
            .write()
            .push_front((AudioCommands::SetPanning(panning), ChannelId::default()));
    }

    pub fn set_pitch(&self, pitch: f32) {
        self.commands
            .write()
            .push_front((AudioCommands::SetPitch(pitch), ChannelId::default()));
    }

    pub fn play_in_channel(&self, audio_source: Handle<AudioSource>, channel_id: &ChannelId) {
        self.commands.write().push_front((
            AudioCommands::Play(PlayAudioSettings {
                source: audio_source,
                looped: false,
            }),
            channel_id.clone(),
        ));
    }

    pub fn play_looped_in_channel(
        &self,
        audio_source: Handle<AudioSource>,
        channel_id: &ChannelId,
    ) {
        self.commands.write().push_front((
            AudioCommands::Play(PlayAudioSettings {
                source: audio_source,
                looped: true,
            }),
            channel_id.clone(),
        ));
    }

    pub fn stop_channel(&self, channel_id: &ChannelId) {
        self.commands
            .write()
            .push_front((AudioCommands::Stop, channel_id.clone()));
    }

    pub fn pause_channel(&self, channel_id: &ChannelId) {
        self.commands
            .write()
            .push_front((AudioCommands::Pause, channel_id.clone()));
    }

    pub fn resume_channel(&self, channel_id: &ChannelId) {
        self.commands
            .write()
            .push_front((AudioCommands::Resume, channel_id.clone()));
    }

    pub fn set_volume_in_channel(&self, volume: f32, channel_id: &ChannelId) {
        self.commands
            .write()
            .push_front((AudioCommands::SetVolume(volume), channel_id.clone()));
    }

    pub fn set_panning_in_channel(&self, panning: f32, channel_id: &ChannelId) {
        self.commands
            .write()
            .push_front((AudioCommands::SetPanning(panning), channel_id.clone()));
    }

    pub fn set_pitch_in_channel(&self, pitch: f32, channel_id: &ChannelId) {
        self.commands
            .write()
            .push_front((AudioCommands::SetPitch(pitch), channel_id.clone()));
    }
}
