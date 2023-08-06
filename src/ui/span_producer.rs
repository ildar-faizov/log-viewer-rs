use std::cmp::{max, min};
use fluent_integer::Integer;
use crate::highlight::style_with_priority::StyleWithPriority;
use crate::utils;

pub struct SpanProducer {
    intervals: Vec<(Integer, Integer, StyleWithPriority)>,
    shift: usize,
    limit: usize,
}

impl SpanProducer {
    pub fn new(shift: usize, limit: usize) -> Self {
        SpanProducer {
            intervals: vec![],
            shift,
            limit,
        }
    }

    pub fn add_interval<A, B>(&mut self, s: A, e: B, style: StyleWithPriority)
        where A: Into<Integer>, B: Into<Integer> {
        self.add_interval_without_shift(s.into() - self.shift, e.into() - self.shift, style)
    }

    pub fn add_interval_without_shift<A, B>(&mut self, s: A, e: B, style: StyleWithPriority)
        where A: Into<Integer>, B: Into<Integer> {
        let s = max(s.into(), 0_u8.into());
        let e = min(e.into(), self.limit.into());
        if s < e {
            self.intervals.push((s, e, style));
        }
    }

    pub fn disjoint_intervals(&self) -> Vec<(Integer, Integer, Vec<StyleWithPriority>)> {
        utils::disjoint_intervals(&self.intervals)
    }
}