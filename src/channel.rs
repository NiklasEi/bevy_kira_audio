#[derive(PartialEq, Eq, Hash, Clone)]
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
