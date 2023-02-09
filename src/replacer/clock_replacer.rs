use crate::{config::FrameId, replacer::replacer::Replacer};
use std::{collections::HashMap, ptr::NonNull};

struct Node {
    frame_id: FrameId,
    ref_: bool,
    prev: Option<NonNull<Node>>,
    next: Option<NonNull<Node>>,
}

impl Node {
    fn new(frame_id: FrameId) -> Self {
        Self {
            frame_id,
            ref_: true,
            prev: None,
            next: None,
        }
    }
}

pub struct ClockReplacer {
    hand: Option<NonNull<Node>>, // current ptr
    map: HashMap<FrameId, NonNull<Node>>,
    capacity: usize,
    marker: std::marker::PhantomData<Node>, // mark lifetime
}

impl ClockReplacer {
    fn detach(&mut self, mut node: NonNull<Node>) {
        if self.size() == 1 {
            unsafe {
                node.as_mut().prev = None;
                node.as_mut().next = None;
            }
            // drop(node);
            return;
        }

        unsafe {
            match node.as_mut().prev {
                Some(mut prev) => {
                    prev.as_mut().next = node.as_ref().next;
                }
                None => {
                    panic!("prev is None")
                }
            }
            match node.as_mut().next {
                Some(mut next) => {
                    next.as_mut().prev = node.as_mut().prev;
                }
                None => {
                    panic!("next is None")
                }
            }
            node.as_mut().prev = None;
            node.as_mut().next = None;
        }
        // drop(node);
    }
}

impl Replacer for ClockReplacer {
    fn new(frame_num: FrameId) -> Self {
        Self {
            hand: None,
            map: HashMap::new(),
            capacity: frame_num,
            marker: std::marker::PhantomData,
        }
    }

    fn victim(&mut self) -> Option<FrameId> {
        loop {
            let mut hand = self.hand?;
            unsafe {
                if hand.as_ref().ref_ {
                    hand.as_mut().ref_ = false;
                    hand = hand.as_ref().next.unwrap();
                    self.hand = Some(hand);
                } else {
                    let frame_id = hand.as_ref().frame_id;
                    self.remove(hand.as_mut().frame_id);
                    return Some(frame_id);
                }
            }
        }
    }

    fn insert(&mut self, frame_id: usize) {
        if let Some(node) = self.map.get(&frame_id) {
            // already in the replacer
            let mut node = *node;
            unsafe {
                node.as_mut().ref_ = true;
            }
            return;
        }

        if self.size() == 0 {
            let node = Box::into_raw(Box::new(Node::new(frame_id)));
            let mut node = unsafe { NonNull::new_unchecked(node) };
            unsafe {
                node.as_mut().prev = Some(node);
                node.as_mut().next = Some(node);
            }
            self.hand = Some(node);
            self.map.insert(frame_id, node);
        } else if self.size() < self.capacity {
            let node = Box::into_raw(Box::new(Node::new(frame_id)));
            let mut node = unsafe { NonNull::new_unchecked(node) };
            let mut hand = self.hand.unwrap();
            unsafe {
                let mut tmp = hand.as_mut().prev.unwrap();
                hand.as_mut().prev = Some(node);
                node.as_mut().next = Some(hand);
                node.as_mut().prev = Some(tmp);
                tmp.as_mut().next = Some(node);
            }
            self.map.insert(frame_id, node);
        } else {
            // full
            loop {
                let mut hand = self.hand.unwrap();
                unsafe {
                    if hand.as_ref().ref_ {
                        hand.as_mut().ref_ = false;
                        hand = hand.as_ref().next.unwrap();
                        self.hand = Some(hand);
                    } else {
                        let old_frame_id = hand.as_ref().frame_id;
                        self.map.remove(&old_frame_id);
                        hand.as_mut().ref_ = true;
                        hand.as_mut().frame_id = frame_id;
                        self.map.insert(frame_id, hand);
                        return;
                    }
                }
            }
        }
    }

    fn remove(&mut self, frame_id: usize) {
        if let Some(node) = self.map.get(&frame_id) {
            let node = *node;
            if self.size() == 1 {
                self.hand = None;
            } else {
                unsafe {
                    if self.hand.unwrap().as_ref().frame_id == frame_id {
                        self.hand = Some(node.as_ref().next.unwrap());
                    }
                }
            }
            self.detach(node);
            self.map.remove(&frame_id);
            drop(node.as_ptr());
        }
    }

    fn print(&self) {
        if let Some(hand) = self.hand {
            unsafe {
                print!("({}, {}) -> ", hand.as_ref().frame_id, hand.as_ref().ref_);
            }
        } else {
            return;
        }

        let hand = self.hand.unwrap();
        unsafe {
            let mut next = hand.as_ref().next.unwrap();
            loop {
                if next.as_ref().frame_id == hand.as_ref().frame_id {
                    break;
                } else {
                    print!("({}, {}) -> ", next.as_ref().frame_id, next.as_ref().ref_);
                    next = next.as_ref().next.unwrap();
                }
            }
        }
        println!();
    }

    fn size(&self) -> usize {
        self.map.len()
    }
}

impl Drop for ClockReplacer {
    fn drop(&mut self) {
        for (_, node) in self.map.iter() {
            drop(node.as_ptr());
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clock_replacer_test1() {
        let mut replacer = ClockReplacer::new(4);
        // 2 5 10 1
        replacer.insert(2);
        replacer.insert(5);
        replacer.insert(10);
        replacer.insert(1);
        replacer.print();
        // 5(false) 10(false) 1(false)
        assert_eq!(replacer.victim(), Some(2));
        replacer.print();
        assert_eq!(replacer.size(), 3);
        // 5(true) 10(false) 1(false)
        replacer.insert(5);
        replacer.print();
        assert_eq!(replacer.victim(), Some(10));
        replacer.print();
        replacer.remove(5);
        replacer.print();
        assert_eq!(replacer.size(), 1);
    }

    #[test]
    fn clock_replacer_test2() {
        let mut replacer = ClockReplacer::new(4);
        replacer.insert(1);
        replacer.insert(2);
        replacer.insert(3);
        replacer.insert(4);
        assert_eq!(replacer.victim(), Some(1));
        assert_eq!(replacer.victim(), Some(2));
        replacer.remove(4);
        assert_eq!(replacer.size(), 1);
        replacer.insert(5);
        replacer.insert(6);
        assert_eq!(replacer.size(), 3);
        replacer.print();
        replacer.remove(6);
        replacer.remove(5);
        replacer.remove(3);
        assert_eq!(replacer.size(), 0);
    }
}
