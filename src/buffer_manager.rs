use std::collections::HashMap;
use std::ops::Not;

use crate::config::*;
use crate::data_storage_manager::DSMgr;
use crate::page::*;
use crate::replacer::*;

pub struct BMgr {
    data_storage_manager: DSMgr,
    // capacity: usize,
    free_list: Vec<usize>,
    pages: Vec<Page>,
    replacer: Box<dyn Replacer>,
    page_table: HashMap<PageId, FrameId>,
    num_write_io: usize,
    num_read_io: usize,
    num_hits: usize,
}

impl BMgr {
    pub fn new(db_file_name: &str, replace_policy: ReplacePolicy, frame_num: usize) -> BMgr {
        let mut free_list = Vec::with_capacity(frame_num);
        let mut pages = Vec::with_capacity(frame_num);
        for i in (0..frame_num).rev() {
            free_list.push(i);
        }
        for _ in 0..frame_num {
            pages.push(Page::new(None));
        }

        let replacer: Box<dyn Replacer> = match replace_policy {
            ReplacePolicy::Lru => Box::new(LruReplacer::new(frame_num)),
            ReplacePolicy::Clock => Box::new(ClockReplacer::new(frame_num)),
        };

        let data_storage_manager = DSMgr::new(db_file_name);

        BMgr {
            data_storage_manager,
            // capacity: frame_num,
            free_list,
            pages,
            replacer,
            page_table: HashMap::new(),
            num_write_io: 0,
            num_read_io: 0,
            num_hits: 0,
        }
    }

    pub fn fix_page(&mut self, page_id: PageId, is_dirty: bool) -> Option<FrameId> {
        if let Some(frame_id) = self.page_table.get(&page_id) {
            self.num_hits += 1;
            let page = &mut self.pages[*frame_id];
            if page.get_pin_count() == 0 {
                self.replacer.remove(*frame_id);
            }
            page.increment_pin_count();
            if is_dirty {
                page.set_dirty(true);
            }
            Some(*frame_id)
        } else {
            if let Some(frame_id) = self.select_victim() {
                let page = &mut self.pages[frame_id];
                // TODO optimize
                page.get_data()
                    .copy_from_slice(&self.data_storage_manager.read_page(page_id).unwrap());
                self.num_read_io += 1;
                self.page_table.insert(page_id, frame_id);
                page.increment_pin_count();
                page.set_page_id(page_id);
                if is_dirty {
                    page.set_dirty(true);
                }
                Some(frame_id)
            } else {
                None
            }
        }
    }

    #[allow(dead_code)]
    pub fn fix_new_page(&mut self, page_id: &mut PageId) -> Option<FrameId> {
        if let Some(frame_id) = self.select_victim() {
            *page_id = self.data_storage_manager.new_page();
            let page = &mut self.pages[frame_id];
            self.page_table.insert(*page_id, frame_id);
            page.increment_pin_count();
            page.set_page_id(*page_id);
            page.set_dirty(true);
            Some(frame_id)
        } else {
            None
        }
    }

    pub fn unfix_page(&mut self, page_id: PageId) -> Option<FrameId> {
        if let Some(frame_id) = self.page_table.get(&page_id) {
            let page = &mut self.pages[*frame_id];
            assert!(page.get_pin_count() > 0);
            page.decrement_pin_count();
            if page.get_pin_count() == 0 {
                self.replacer.insert(*frame_id);
            }
            Some(*frame_id)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn num_free_frames(&self) -> usize {
        self.free_list.len()
    }

    pub fn select_victim(&mut self) -> Option<FrameId> {
        if self.free_list.is_empty().not() {
            return self.free_list.pop();
        }

        let victim_frame_id: FrameId = self.replacer.victim()?;
        let page = &mut self.pages[victim_frame_id];
        assert_eq!(page.get_pin_count(), 0);
        if page.is_dirty() {
            self.data_storage_manager.write_page(page).unwrap();
            self.num_write_io += 1;
            page.set_dirty(false);
        }
        self.page_table.remove(&page.get_page_id().unwrap());
        Some(victim_frame_id)
    }

    #[allow(dead_code)]
    fn set_dirty(&mut self, frame_id: FrameId) {
        let page = &mut self.pages[frame_id];
        page.set_dirty(true);
    }

    #[allow(dead_code)]
    fn unset_dirty(&mut self, frame_id: FrameId) {
        let page = &mut self.pages[frame_id];
        page.set_dirty(false);
    }

    fn write_dirtys(&mut self) {
        for page in &mut self.pages {
            if page.is_dirty() {
                self.data_storage_manager.write_page(page).unwrap();
                self.num_write_io += 1;
                page.set_dirty(false);
            }
        }
    }

    pub fn get_read_io_num(&self) -> usize {
        self.num_read_io
    }

    pub fn get_write_io_num(&self) -> usize {
        self.num_write_io
    }

    pub fn get_io_num(&self) -> usize {
        self.num_read_io + self.num_write_io
    }

    pub fn get_hit_num(&self) -> usize {
        self.num_hits
    }

    #[allow(dead_code)]
    pub fn print_page_table(&self) {
        println!("Page Table:");
        for (page_id, frame_id) in &self.page_table {
            println!("page_id: {:?}, frame_id: {:?}", page_id, frame_id);
        }
        println!();
    }

    #[allow(dead_code)]
    pub fn print_replacer(&self) {
        self.replacer.print();
    }

    pub fn print_frame(&self, frame_id: FrameId) {
        let page = &self.pages[frame_id];
        println!("Frame: {:?}", frame_id);
        println!("page_id: {:?}", page.get_page_id());
        println!("pin_count: {:?}", page.get_pin_count());
        println!("is_dirty: {:?}", page.is_dirty());
        println!();
    }

    #[allow(dead_code)]
    pub fn print_all_frames(&self) {
        for i in 0..self.pages.len() {
            self.print_frame(i);
        }
    }
}

impl Drop for BMgr {
    fn drop(&mut self) {
        self.write_dirtys();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_buffer_manager1() {
        let file = format!("./target/test_file_{:?}.dbf", std::thread::current().id());
        let mut buffer_manager = BMgr::new(&file, ReplacePolicy::Lru, 5);
        let mut page_id = 0;
        for i in 0..5 {
            let _ = buffer_manager.fix_new_page(&mut page_id).unwrap();
            buffer_manager.unfix_page(page_id);
            assert_eq!(page_id, i)
        }
        // 0 1 2 3 4
        for i in 5..10 {
            assert_eq!(buffer_manager.fix_new_page(&mut page_id).unwrap(), i - 5);
            assert_eq!(page_id, i);
            buffer_manager.unfix_page(page_id);
        }

        buffer_manager.print_page_table();
        buffer_manager.print_replacer();

        buffer_manager.fix_page(8, false);
        buffer_manager.unfix_page(8);
        buffer_manager.fix_page(7, false);
        buffer_manager.unfix_page(7);
        buffer_manager.fix_page(5, false);
        buffer_manager.unfix_page(5);
        buffer_manager.print_replacer();

        let expect = vec![1, 4, 3, 2, 0];
        for i in 0..5 {
            assert_eq!(expect[i], buffer_manager.select_victim().unwrap());
        }

        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn test_buffer_manager2() {
        let file = format!("./target/test_file_{:?}.dbf", std::thread::current().id());
        let mut buffer_manager = BMgr::new(&file, ReplacePolicy::Lru, 5);

        // new 30 pages == 120k
        let mut page_id = 0;
        for i in 0..30 {
            let _ = buffer_manager.fix_new_page(&mut page_id).unwrap();
            buffer_manager.unfix_page(page_id);
            assert_eq!(page_id, i)
        }

        println!("{}", buffer_manager.get_io_num());
        println!("{}", buffer_manager.get_read_io_num());
        println!("{}", buffer_manager.get_write_io_num());
        println!("{}", buffer_manager.get_hit_num());
        println!();

        for i in 0..5 {
            assert_eq!(buffer_manager.fix_page(i, false).unwrap(), i);
            buffer_manager.unfix_page(i);
        }

        assert_eq!(buffer_manager.get_read_io_num(), 5);
        assert_eq!(buffer_manager.get_hit_num(), 0);

        for i in (0..5).rev() {
            assert_eq!(buffer_manager.fix_page(i, false).unwrap(), i);
            buffer_manager.unfix_page(i);
        }

        let last_write_io = buffer_manager.get_write_io_num();

        // replacer frame: 4 3 2 1 0
        buffer_manager.print_replacer();
        assert_eq!(buffer_manager.get_read_io_num(), 5);
        assert_eq!(buffer_manager.get_hit_num(), 5);
        buffer_manager.set_dirty(4);
        buffer_manager.set_dirty(3);
        assert_eq!(buffer_manager.fix_page(5, false).unwrap(), 4);
        buffer_manager.unfix_page(5);
        assert_eq!(buffer_manager.fix_page(6, false).unwrap(), 3);
        buffer_manager.unfix_page(6);
        assert_eq!(buffer_manager.get_read_io_num(), 7);
        assert_eq!(buffer_manager.get_write_io_num() - last_write_io, 2);

        let _ = std::fs::remove_file(file);
    }
}
