use std::cmp::Ordering;
use fluent_integer::Integer;

#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub start: Integer,
    pub end: Integer
}

impl Selection {
    pub fn new(start: Integer, end: Integer) -> Self {
        Selection { start, end }
    }

    pub fn create(boundary1: Integer, boundary2: Integer) -> Option<Box<Selection>> {
        match boundary1.cmp(&boundary2) {
            Ordering::Less => Some(Box::new(Selection::new(boundary1, boundary2))),
            Ordering::Equal => None,
            Ordering::Greater => Some(Box::new(Selection::new(boundary2, boundary1))),
        }
    }
}