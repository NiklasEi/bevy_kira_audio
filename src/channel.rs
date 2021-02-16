#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ChannelId {
    key: String,
}

impl Default for ChannelId {
    fn default() -> Self {
        ChannelId {
            key: "default_channel".to_string(),
        }
    }
}

impl ChannelId {
    pub fn new(key: String) -> Self {
        ChannelId { key }
    }
}
