//! Animation engine for creature sprites (stub)

/// An animation sequence.
#[derive(Debug, Clone)]
pub struct Animation {
    pub frames: Vec<usize>,
    pub frame_duration_ms: u64,
}

impl Animation {
    pub fn current_frame(&self, elapsed_ms: u64) -> usize {
        if self.frames.is_empty() {
            return 0;
        }
        let total = self.frame_duration_ms * self.frames.len() as u64;
        let pos = elapsed_ms % total;
        self.frames[(pos / self.frame_duration_ms) as usize % self.frames.len()]
    }
}
