use crate::data_source::filtered::filtered_line_source::LineFilter;
use crate::data_source::line_source_holder::ConcreteLineSourceHolder;
use crate::data_source::{CustomHighlight, Direction, Line};
use fluent_integer::Integer;
use std::collections::BTreeMap;

type FilterResult = (Line, Vec<CustomHighlight>); // local name for brevity

pub struct CachingFilter {
    filter: LineFilter,
    cache_size: usize,
    cache: BTreeMap<Integer, FilterResult>,
}

impl CachingFilter {
    pub fn new(
        filter: LineFilter,
        cache_size: usize
    ) -> Self {
        Self {
            filter,
            cache_size,
            cache: BTreeMap::new(),
        }
    }

    pub fn apply(
        &mut self,
        origin: &mut ConcreteLineSourceHolder,
        offset: Integer,
        direction: Direction
    ) -> Option<FilterResult> {
        let mut sub_tree = self.cache.range(..=offset);
        let mut read: Box<dyn FnMut(Integer) -> Option<Line>> = match direction {
            Direction::Forward => Box::new(|i: Integer| origin.read_next_line(i)),
            Direction::Backward => Box::new(|i: Integer| origin.read_prev_line(i)),
        };
        if let Some((_, entry)) = sub_tree.next_back() {
            if entry.0.end >= offset {
                Some(entry.clone())
            } else {
                let line = read(offset)?;
                if line.start - entry.0.end > 1 {
                    self.cache.clear();
                }
                let filter_result = (self.filter)(&line.content);
                if self.cache.len() + 1 >= self.cache_size {
                    self.cache.pop_first();
                }
                self.cache.insert(line.start, (line.clone(), filter_result.clone()));
                Some((line, filter_result))
            }
        } else {
            let line = read(offset)?;
            let filter_result = (self.filter)(&line.content);
            if let Some((key, _)) = self.cache.first_key_value() {
                if line.end + 1 < key {
                    self.cache.clear();
                }
            }
            if self.cache.len() + 1 >= self.cache_size {
                self.cache.pop_last();
            }
            self.cache.insert(line.start, (line.clone(), filter_result.clone()));
            Some((line, filter_result))
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO write tests, verify origin is not invoked more than necessary
}
