use std::time::Instant;

use crate::{replacer::replacer::Replacer, config::FrameId};

#[derive(Debug)]
struct PageMetadata {
    frame_id: FrameId,
    last_accessed_at: Instant,
}

impl PageMetadata {
    pub fn new(frame_id: usize) -> Self {
        Self {
            frame_id,
            last_accessed_at: Instant::now(),
        }
    }
}

pub struct LruReplacer {
    page_table: Vec<PageMetadata>,
}

impl Replacer for LruReplacer {
    fn new(frame_num: FrameId) -> Self {
        LruReplacer {
            page_table: Vec::with_capacity(frame_num),
        }
    }

    fn victim(&mut self) -> Option<FrameId> {
        self.page_table.sort_by(|a, b| b.last_accessed_at.cmp(&a.last_accessed_at));
        match self.page_table.pop() {
            Some(page) => Some(page.frame_id),
            None => None,
        }
    }

    fn insert(&mut self, frame_id: usize) {
        self.page_table.push(PageMetadata::new(frame_id));
    }

    fn remove(&mut self, frame_id: usize) {
        if let Some(index) = self.page_table.iter().position(|md| md.frame_id == frame_id) {
            self.page_table.remove(index);
        }
    }

    fn print(&self) {

    }

    fn size(&self) -> usize {
        self.page_table.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lru_replacer_test() {
        let mut replacer = LruReplacer::new(4);

        // We have 3 candidates that can be choose to
        // be evicted by our buffer pool.
        replacer.insert(2);
        sleep(5);
        replacer.insert(0);
        sleep(5);
        replacer.insert(1);

        let evicted_frame_id = replacer.victim().unwrap();
        assert_eq!(evicted_frame_id, 2);
    }

    #[test]
    fn lru_replacer_test2() {
        let mut replacer = LruReplacer::new(4);

        // We have 3 candidates that can be choose to
        // be evicted by our buffer pool.
        replacer.insert(2);
        sleep(5);
        replacer.insert(0);
        sleep(5);
        replacer.insert(1);
        replacer.remove(2);

        let evicted_page_id = replacer.victim().unwrap();
        assert_eq!(evicted_page_id, 0);
    }

    fn sleep(duration_in_ms: u64) {
        let ten_millis = std::time::Duration::from_millis(duration_in_ms);
        std::thread::sleep(ten_millis);
    }

}

