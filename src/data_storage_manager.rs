use crate::{config::*, page::*};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write}
};

pub struct DSMgr {
    curr_file: File,
    num_pages: usize,
}

impl DSMgr {
    pub fn new(path: &str) -> DSMgr {
        let curr_file = Self::open_file(&path);
        let file_len = curr_file.metadata().unwrap().len() as usize;
        let num_pages = file_len / PAGE_SIZE;

        DSMgr {
            curr_file,
            num_pages,
        }
    }

    fn open_file(path: &str) -> File {
        let curr_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .unwrap();
        curr_file
    }

    #[allow(dead_code)]
    fn close_file(&self) {
        // due to rust ownership mechanism, we don't need to implement close_file
        unimplemented!()
    }

    pub fn read_page(&mut self, page_id: usize) -> Result<Data, std::io::Error> {
        let offset = page_id * PAGE_SIZE;

        self.seek(offset);
        let mut buffer: Data = [0; PAGE_SIZE];
        self.curr_file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn write_page(&mut self, page: &mut Page) -> Result<(), std::io::Error> {
        let page_id = page.get_page_id().unwrap();
        let offset = page_id * PAGE_SIZE;
        let data = page.get_data();

        self.seek(offset);
        self.curr_file.write_all(data)?;
        self.curr_file.flush()
    }

    fn seek(&mut self, offset: usize) {
        self.curr_file
            .seek(std::io::SeekFrom::Start(offset as u64))
            .expect("cannot seek");
    }

    #[allow(dead_code)]
    pub fn get_file(&self) -> &File {
        &self.curr_file
    }

    pub fn inc_num_pages(&mut self) {
        self.num_pages += 1;
    }

    #[allow(dead_code)]
    pub fn get_num_pages(&self) -> usize {
        self.num_pages
    }

    #[allow(unused_variables, dead_code)]
    pub fn set_use(page_id: PageId, use_bit: i32) {
        // don't need
        unimplemented!()
    }

    #[allow(unused_variables, dead_code)]
    pub fn get_use(page_id: PageId) -> i32 {
        // don't need
        unimplemented!()
    }

    pub fn new_page(&mut self) -> PageId {
        let buffer = [0; PAGE_SIZE];
        let new_page_id = self.num_pages;
        self.inc_num_pages();
        let offset = new_page_id * PAGE_SIZE;
        self.seek(offset);
        self.curr_file.write_all(&buffer).unwrap();
        self.curr_file.flush().unwrap();
        new_page_id
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_data_storage_manager() {
        let file = format!("./target/test_file_{:?}.dbf", std::thread::current().id());
        let mut disk_manager = DSMgr::new(&file);

        for i in 0..100 {
            let page_id = disk_manager.new_page();
            assert_eq!(page_id, i);
            let mut page = Page::new(Some(i));
            let test_data = format!("test data: {}", i);
            page.get_data()[..test_data.len()].copy_from_slice(test_data.as_bytes());
            disk_manager.write_page(&mut page).unwrap();
        }
        assert_eq!(disk_manager.get_num_pages(), 100);

        for i in 0..100 {
            let test_data = disk_manager.read_page(i).unwrap();
            let test_target_data = format!("test data: {}", i);
            assert_eq!(&test_data[..test_target_data.len()], test_target_data.as_bytes());
        }
        assert_eq!(disk_manager.get_num_pages(), 100);
        let _ = std::fs::remove_file(file);
    }
}
