use fluent_integer::Integer;
use crate::data_source::Direction;
use crate::interval::{Interval, IntervalBound};
use crate::search::navigable_searcher::NavigableSearcher;
use crate::search::searcher::{Occurrence, Searcher, SearchError, SearchResult};

pub struct NavigableSearcherImpl {
    cache: Vec<Occurrence>,
    range: Interval<Integer>,
    searcher: Box<dyn Searcher>,
}

impl NavigableSearcherImpl {
    pub fn new(searcher: Box<dyn Searcher>) -> Self {
        NavigableSearcherImpl {
            cache: vec![],
            range: Interval::empty(),
            searcher
        }
    }
}

impl NavigableSearcher for NavigableSearcherImpl {
    fn find_all_in_range(&mut self, range: Interval<Integer>) -> Result<Vec<Occurrence>, SearchError> {
        // if ranges are same, nothing to do
        if range == self.range {
            return Ok(self.cache.clone())
        }

        // TODO: optimize to reuse occurrences if ranges intersect

        let mut new_occurrences = vec![];
        self.range = range;
        let mut scope = range;
        loop {
            match self.searcher.search(Direction::Forward, scope) {
                Ok(p) => {
                    new_occurrences.push(p);
                    scope = scope.to_builder().left_bound_exclusive(p.start).build()
                }
                Err(SearchError::NotFound) => break,
                Err(e) => return Err(e)
            }
        }
        self.cache = new_occurrences.clone();
        Ok(new_occurrences)
    }

    fn next_occurrence(&mut self, direction: Direction) -> SearchResult {
        // TODO: replace with interval subtraction
        let scope = match direction {
            Direction::Forward => {
                match self.range.right_bound {
                    IntervalBound::NegativeInfinity => Interval::all(),
                    IntervalBound::PositiveInfinity => Interval::empty(),
                    IntervalBound::Fixed { value, is_included } =>
                        Interval::builder().left_bound(value, !is_included).right_unbounded().build(),
                }
            },
            Direction::Backward => {
                match self.range.left_bound {
                    IntervalBound::NegativeInfinity => Interval::empty(),
                    IntervalBound::PositiveInfinity => Interval::all(),
                    IntervalBound::Fixed { value, is_included } =>
                        Interval::builder().left_unbounded().right_bound(value, !is_included).build(),
                }
            }
        };
        self.searcher.search(direction, scope)
    }

    fn set_initial_offset(&mut self, offset: Integer) {
        self.range = Interval::point(offset);
    }
}

// Tests are included according to http://xion.io/post/code/rust-unit-test-placement.html
#[cfg(test)]
#[path = "./navigable_searcher_tests.rs"]
mod navigable_searcher_tests;