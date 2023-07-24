// image_formats::image
// by Desmond Germans, 2019

use std::collections::{HashMap, VecDeque};

#[derive(Default, Clone)]
pub struct ImageBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
}

impl ImageBuffer {
    pub fn new(width: usize, height: usize) -> ImageBuffer {
        ImageBuffer {
            width,
            height,
            data: vec![0; width * height],
        }
    }
}

pub struct LRUCache {
    map: HashMap<String, ImageBuffer>,
    order: VecDeque<String>,
    capacity: usize,
}

impl LRUCache {
    pub fn new(capacity: usize) -> Self {
        LRUCache {
            capacity,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<ImageBuffer> {
        if self.map.contains_key(key) {
            self.order.retain(|k| *k != key.to_string());
            self.order.push_back(key.to_string());
            self.map.get(key).cloned()
        } else {
            None
        }
    }

    pub fn put(&mut self, key: &str, value: ImageBuffer) {
        if self.map.len() == self.capacity {
            if let Some(lru_key) = self.order.pop_front() {
                self.map.remove(&lru_key);
            }
        }
        self.order.push_back(key.to_string());
        self.map.insert(key.to_string(), value);
    }
}
