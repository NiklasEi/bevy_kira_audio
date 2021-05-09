use std::fmt::Debug;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Frame {
    pub left: f32,
    pub right: f32,
}

impl Into<kira::Frame> for Frame {
    fn into(self) -> kira::Frame {
        kira::Frame {
            left: self.left,
            right: self.right,
        }
    }
}

pub trait AudioStream: Debug + Send + Sync {
    fn next(&mut self, dt: f64) -> Frame;
}
