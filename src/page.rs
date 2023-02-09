use std::usize;

use crate::config::*;

pub struct Page {
    page_id: Option<PageId>,
    data: Data,
    is_dirty: bool,
    pin_count: usize,
}

impl Page {
    pub fn new(page_id: Option<PageId>) -> Page {
        Page {
            page_id,
            data: [0; PAGE_SIZE],
            is_dirty: false,
            pin_count: 0,
        }
    }

    pub fn get_page_id(&self) -> Option<PageId> {
        self.page_id
    }

    pub fn set_page_id(&mut self, page_id: PageId)  {
        self.page_id = Some(page_id);
    }

    pub fn get_data(&mut self) -> &mut Data {
        &mut self.data
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }

    pub fn get_pin_count(&self) -> usize {
        self.pin_count
    }

    pub fn increment_pin_count(&mut self) {
        self.pin_count += 1;
    }

    pub fn decrement_pin_count(&mut self) {
        self.pin_count -= 1;
    }
}