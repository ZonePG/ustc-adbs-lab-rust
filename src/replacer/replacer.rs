use crate::config::FrameId;

pub trait Replacer {
    fn victim(&mut self) -> Option<FrameId>;
    fn insert(&mut self, frame_id: usize);
    fn remove(&mut self, frame_id: usize);
    fn print(&self);
    fn size(&self) -> usize;
}
