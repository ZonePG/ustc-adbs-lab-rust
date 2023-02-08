use crate::{replacer::replacer::Replacer, config::FrameId};


#[derive(Debug)]
struct PageMetadata {
    frame_id: FrameId,
    is_ref: bool,
}

impl PageMetadata {
    pub fn new(frame_id: usize) -> Self {
        Self {
            frame_id,
            is_ref: true,
        }
    }
}

struct ClockReplacer {
    page_table: Vec<PageMetadata>,
    hand: usize,
}

impl Replacer for ClockReplacer {
    fn new(frame_num: FrameId) -> Self {
        ClockReplacer {
            page_table: Vec::with_capacity(frame_num),
            hand: 0,
        }
    }

    fn victim(&mut self) -> Option<FrameId> {
        if self.page_table.is_empty() {
            return None;
        }

        let mut victim = None;
        loop {
            if self.page_table[self.hand].is_ref {
                self.page_table[self.hand].is_ref = false;
            } else {
                victim = Some(self.page_table[self.hand].frame_id);
                self.page_table.remove(self.hand);
                break;
            }
            self.hand = (self.hand + 1) % self.page_table.len();
        }
        victim
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
    fn clock_replacer_test() {
        let mut replacer = ClockReplacer::new(4);

        // We have 3 candidates that can be choose to
        // be evicted by our buffer pool.
        replacer.insert(2);
        replacer.insert(5);
        replacer.insert(10);
        replacer.insert(1);
        assert_eq!(replacer.victim(), Some(2));
        assert_eq!(replacer.size(), 3);
        replacer.remove(5);
        assert_eq!(replacer.size(), 2);
    }

    #[test]
    fn clock_replacer_test2() {
        let mut replacer = ClockReplacer::new(4);

        // We have 3 candidates that can be choose to
        // be evicted by our buffer pool.
        replacer.insert(2);
        replacer.insert(0);
        replacer.insert(1);
        replacer.remove(2);

        let evicted_page_id = replacer.victim().unwrap();
        assert_eq!(evicted_page_id, 0);
    }
}


