use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CustomHighlight {
    start: usize,
    end: usize,
}

pub type CustomHighlights = HashMap<&'static str, Vec<CustomHighlight>>;

impl CustomHighlight {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}