use parking_lot::RwLock;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Audio {
    pub queue: RwLock<VecDeque<String>>,
    pub loop_queue: RwLock<VecDeque<String>>,
}

impl Audio {
    pub fn play(&self, path: String) {
        self.queue.write().push_front(path);
    }
    pub fn play_loop(&self, path: String) {
        self.loop_queue.write().push_front(path);
    }
}
