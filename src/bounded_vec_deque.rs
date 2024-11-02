use std::collections::VecDeque;

pub struct BoundedVecDeque<T> {
    vec: VecDeque<T>,
    capacity: usize,
}

impl<T> BoundedVecDeque<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        BoundedVecDeque {
            vec: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push_back(&mut self, value: T) -> Option<T> {
        let extra = if self.vec.len() == self.capacity {
            self.vec.pop_front()
        } else {
            None
        };
        self.vec.push_back(value);
        extra
    }

    pub fn push_front(&mut self, value: T) -> Option<T> {
        let extra = if self.vec.len() == self.capacity {
            self.vec.pop_back()
        } else {
            None
        };
        self.vec.push_front(value);
        extra
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.vec.pop_front()
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.vec.pop_back()
    }

    pub fn front(&self) -> Option<&T> {
        self.vec.front()
    }

    pub fn back(&self) -> Option<&T> {
        self.vec.back()
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }
}