use std::{collections::HashMap, ptr::NonNull};

use crate::{config::FrameId, replacer::replacer::Replacer};

struct Node {
    frame_id: FrameId,
    prev: Option<NonNull<Node>>,
    next: Option<NonNull<Node>>,
}

impl Node {
    fn new(frame_id: FrameId) -> Self {
        Self {
            frame_id,
            prev: None,
            next: None,
        }
    }
}

pub struct LruReplacer {
    head: Option<NonNull<Node>>, // head -> LRU
    tail: Option<NonNull<Node>>, // tail -> MRU
    map: HashMap<FrameId, NonNull<Node>>,
    capacity: usize,
    marker: std::marker::PhantomData<Node>, // mark lifetime
}

impl LruReplacer {
    pub fn new(frame_num: usize) -> Self {
        Self {
            head: None,
            tail: None,
            map: HashMap::new(),
            capacity: frame_num,
            marker: std::marker::PhantomData,
        }
    }


    fn detach(&mut self, mut node: NonNull<Node>) {
        // delete specific node
        unsafe {
            match node.as_mut().prev {
                Some(mut prev) => {
                    prev.as_mut().next = node.as_ref().next;
                }
                None => {
                    self.head = node.as_ref().next;
                }
            }
            match node.as_mut().next {
                Some(mut next) => {
                    next.as_mut().prev = node.as_mut().prev;
                }
                None => {
                    self.tail = node.as_ref().prev;
                }
            }

            node.as_mut().prev = None;
            node.as_mut().next = None;
        }
    }

    fn attach(&mut self, mut node: NonNull<Node>) {
        match self.tail {
            Some(mut tail) => unsafe {
                tail.as_mut().next = Some(node);
                node.as_mut().prev = Some(tail);
                node.as_mut().next = None;
                self.tail = Some(node);
            },
            None => {
                unsafe {
                    node.as_mut().prev = None;
                    node.as_mut().next = None;
                }
                self.head = Some(node);
                self.tail = Some(node);
            }
        }
    }
}

impl Replacer for LruReplacer {
    fn victim(&mut self) -> Option<FrameId> {
        let head = self.head?;
        let victim_frame_id = unsafe { head.as_ref().frame_id };
        self.detach(head);
        self.map.remove(&victim_frame_id);
        drop(head.as_ptr());
        Some(victim_frame_id)
    }

    fn insert(&mut self, frame_id: usize) {
        // remove old node if exists
        if let Some(node) = self.map.get(&frame_id) {
            let node = *node;
            self.detach(node);
            self.attach(node);
        } else {
            let node = Box::into_raw(Box::new(Node::new(frame_id)));
            let node = unsafe { NonNull::new_unchecked(node) };
            self.map.insert(frame_id, node);
            self.attach(node);
            if self.map.len() > self.capacity {
                self.victim();
            }
        }
    }

    fn remove(&mut self, frame_id: usize) {
        if let Some(node) = self.map.get(&frame_id) {
            let node = *node;
            self.detach(node);
            self.map.remove(&frame_id);
            drop(node.as_ptr());
        }
    }

    fn print(&self) {
        print!("Lru replacer: ");
        let mut node = self.head;
        while let Some(n) = node {
            unsafe {
                print!("{} ", n.as_ref().frame_id);
                node = n.as_ref().next;
            }
        }
        println!();
    }

    fn size(&self) -> usize {
        self.map.len()
    }
}

impl Drop for LruReplacer {
    fn drop(&mut self) {
        while let Some(node) = self.head.take() {
            unsafe {
                self.head = node.as_ref().next;
                drop(node.as_ptr());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lru_replacer_test() {
        let mut replacer = LruReplacer::new(5);
        // 1 2 3 4 5
        for i in 1..=5 {
            replacer.insert(i);
            assert_eq!(replacer.size(), i);
        }
        replacer.insert(1);
        // 2 3 4 5 1
        assert_eq!(replacer.size(), 5);
        // 3 4 5 1
        assert_eq!(replacer.victim().unwrap(), 2);
        assert_eq!(replacer.size(), 4);
        // 3 5 1
        replacer.remove(4);
        assert_eq!(replacer.size(), 3);
        // 5 1 7 8 9
        for i in 7..=9 {
            replacer.insert(i);
        }
        // 7 8 9 5 1
        replacer.insert(5);
        replacer.insert(1);
        assert_eq!(replacer.victim().unwrap(), 7);
        assert_eq!(replacer.victim().unwrap(), 8);
        assert_eq!(replacer.victim().unwrap(), 9);
        assert_eq!(replacer.victim().unwrap(), 5);
        assert_eq!(replacer.victim().unwrap(), 1);
        assert_eq!(replacer.size(), 0);
        for i in 10..15 {
            replacer.insert(i);
        }
        assert_eq!(replacer.size(), 5);
    }
}
